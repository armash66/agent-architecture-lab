use bevy::prelude::*;
use std::collections::HashMap;
use crate::engine::world::{Grid, Position};
use crate::agents::fsm::FSMAgent;
use crate::agents::astar::AStarAgent;
use crate::agents::behavior_tree::BehaviorTreeAgent;
use super::resources::{SimState, HeatmapMaterials};
use super::components::{AgentKind, AgentMarker, OrbitCamera, GoalMarker};

// Constants replicated for setup. Ideally these should be in a shared config or passed in.
const GRID_W: usize = 12;
const GRID_H: usize = 8;
const CELL_SIZE: f32 = 1.0;
const AGENT_Y: f32 = 0.35;
const OBSTACLE_DENSITY: f32 = 0.15;

fn grid_to_world(pos: Position, y_offset: f32) -> Vec3 {
    Vec3::new(
        pos.x as f32 * CELL_SIZE,
        y_offset,
        pos.y as f32 * CELL_SIZE,
    )
}

fn agent_color(kind: AgentKind) -> Color {
    match kind {
        AgentKind::Fsm => Color::srgb(0.2, 0.8, 0.4),        // green
        AgentKind::AStar => Color::srgb(0.3, 0.5, 1.0),       // blue
        AgentKind::BehaviorTree => Color::srgb(1.0, 0.4, 0.2), // orange
    }
}

fn agent_y_offset(kind: AgentKind) -> f32 {
    match kind {
        AgentKind::Fsm => AGENT_Y,
        AgentKind::AStar => AGENT_Y + 0.01,
        AgentKind::BehaviorTree => AGENT_Y + 0.02,
    }
}

pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let goal = Position {
        x: GRID_W - 1,
        y: GRID_H - 1,
    };
    let mut grid = Grid::new(GRID_W, GRID_H, goal);
    grid.scatter_obstacles(OBSTACLE_DENSITY);

    // ── Materials ──────────────────────────
    let default_light = materials.add(StandardMaterial {
        base_color: Color::srgb(0.85, 0.85, 0.85),
        ..default()
    });
    let default_dark = materials.add(StandardMaterial {
        base_color: Color::srgb(0.65, 0.65, 0.70),
        ..default()
    });

    let fsm_visited = materials.add(StandardMaterial {
        base_color: Color::srgb(0.6, 0.9, 0.7), 
        ..default()
    });
    let astar_visited = materials.add(StandardMaterial {
        base_color: Color::srgb(0.6, 0.7, 0.95), 
        ..default()
    });
    let bt_visited = materials.add(StandardMaterial {
        base_color: Color::srgb(0.95, 0.7, 0.6), 
        ..default()
    });
    let multi_visited = materials.add(StandardMaterial {
        base_color: Color::srgb(0.6, 0.5, 0.6), 
        ..default()
    });

    commands.insert_resource(HeatmapMaterials {
        default_light: default_light.clone(),
        default_dark: default_dark.clone(),
        fsm_visited,
        astar_visited,
        bt_visited,
        multi_visited,
    });

    let cell_mesh = meshes.add(Cuboid::new(CELL_SIZE * 0.95, 0.05, CELL_SIZE * 0.95));

    let mut grid_tile_entities = vec![vec![Entity::PLACEHOLDER; GRID_W]; GRID_H];

    for y in 0..GRID_H {
        for x in 0..GRID_W {
            let mat = if (x + y) % 2 == 0 {
                default_light.clone()
            } else {
                default_dark.clone()
            };
            let id = commands.spawn((
                Mesh3d(cell_mesh.clone()),
                MeshMaterial3d(mat),
                Transform::from_xyz(x as f32 * CELL_SIZE, 0.0, y as f32 * CELL_SIZE),
            )).id();
            grid_tile_entities[y][x] = id;
        }
    }

    // ── Goal marker ─────────────────────────────────────
    commands.spawn((
        Mesh3d(meshes.add(Cylinder::new(0.3, 0.6))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 0.85, 0.0),
            emissive: bevy::color::LinearRgba::new(2.0, 1.7, 0.0, 1.0),
            ..default()
        })),
        Transform::from_xyz(
            goal.x as f32 * CELL_SIZE,
            0.35,
            goal.y as f32 * CELL_SIZE,
        ),
        GoalMarker,
    ));

    // ── Agent cubes ─────────────────────────────────────
    let agent_mesh = meshes.add(Cuboid::new(0.4, 0.4, 0.4));
    for kind in [AgentKind::Fsm, AgentKind::AStar, AgentKind::BehaviorTree] {
        let color = agent_color(kind);
        let pos = Position { x: 0, y: 0 };
        commands.spawn((
            Mesh3d(agent_mesh.clone()),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: color,
                ..default()
            })),
            Transform::from_translation(grid_to_world(pos, agent_y_offset(kind))),
            AgentMarker { kind },
            Visibility::Visible,
        ));
    }

    // ── Light ───────────────────────────────────────────
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            intensity: 4_000_000.0,
            range: 60.0,
            ..default()
        },
        Transform::from_xyz(
            GRID_W as f32 * 0.5,
            14.0,
            GRID_H as f32 * 0.5,
        ),
    ));

    // ── Orbital camera ──────────────────────────────────
    let focus = Vec3::new(
        (GRID_W as f32 - 1.0) * 0.5,
        0.0,
        (GRID_H as f32 - 1.0) * 0.5,
    );
    let radius = 14.0;
    let yaw: f32 = -0.6;
    let pitch: f32 = 0.7;

    let cam_pos = focus + Vec3::new(
        radius * pitch.cos() * yaw.sin(),
        radius * pitch.sin(),
        radius * pitch.cos() * yaw.cos(),
    );
    commands.spawn((
        Camera3d::default(),
        Transform::from_translation(cam_pos).looking_at(focus, Vec3::Y),
        OrbitCamera {
            focus,
            radius,
            yaw,
            pitch,
        },
    ));

    // ── Simulation state ────────────────────────────────
    let fsm = FSMAgent::with_config(0, 0, 0.15, 10, 0.995);
    let astar = AStarAgent::with_config(0, 0, Some(30), 0.1, 10, 0.995);
    let bt = BehaviorTreeAgent::with_config(0, 0, 0.15, 10, 0.995);

    commands.insert_resource(SimState {
        grid,
        fsm,
        astar,
        bt,
        tick_timer: 0.0,
        total_ticks: 0,
        fsm_done: false,
        astar_done: false,
        bt_done: false,
        all_done_printed: false,
        cell_visitors: HashMap::new(),
        grid_tile_entities,
    });
}
