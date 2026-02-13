//! 3D viewer for the Cognitive Grid simulation using Bevy 0.15.
//!
//! Run with: `cargo run --bin viewer`

use bevy::prelude::*;

use cognitive_grid::agents::fsm::{FSMAgent, FSMState};
use cognitive_grid::agents::astar::AStarAgent;
use cognitive_grid::agents::behavior_tree::BehaviorTreeAgent;
use cognitive_grid::engine::world::{Grid, Position};

// ─── Constants ──────────────────────────────────────────────
const GRID_W: usize = 10;
const GRID_H: usize = 5;
const CELL_SIZE: f32 = 1.0;
const AGENT_Y: f32 = 0.35;
const TICK_INTERVAL: f32 = 0.25; // seconds between simulation ticks

// ─── Components / Resources ────────────────────────────────
#[derive(Component)]
struct AgentMarker {
    kind: AgentKind,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum AgentKind {
    Fsm,
    AStar,
    BehaviorTree,
}

#[derive(Component)]
struct TrailDot;

#[derive(Component)]
struct GoalMarker;

#[derive(Resource)]
struct SimState {
    grid: Grid,
    fsm: FSMAgent,
    astar: AStarAgent,
    bt: BehaviorTreeAgent,
    tick_timer: Timer,
    total_ticks: u32,
    fsm_done: bool,
    astar_done: bool,
    bt_done: bool,
    all_done_printed: bool,
}

impl SimState {
    fn is_all_done(&self) -> bool {
        self.fsm_done && self.astar_done && self.bt_done
    }
}

// ─── Helpers ────────────────────────────────────────────────
fn grid_to_world(pos: Position) -> Vec3 {
    Vec3::new(
        pos.x as f32 * CELL_SIZE,
        AGENT_Y,
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

fn agent_label(kind: AgentKind) -> &'static str {
    match kind {
        AgentKind::Fsm => "FSM",
        AgentKind::AStar => "A*",
        AgentKind::BehaviorTree => "BT",
    }
}

// ─── Main ───────────────────────────────────────────────────
fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Cognitive Grid — 3D Viewer".to_string(),
                resolution: (1280.0, 720.0).into(),
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup)
        .add_systems(Update, (tick_simulation, sync_agents, rotate_goal))
        .run();
}

// ─── Setup ──────────────────────────────────────────────────
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let goal = Position {
        x: GRID_W - 1,
        y: GRID_H - 1,
    };
    let grid = Grid::new(GRID_W, GRID_H, goal);

    // Ground plane — one quad per cell for a checkerboard effect.
    let cell_mesh = meshes.add(Cuboid::new(CELL_SIZE * 0.95, 0.05, CELL_SIZE * 0.95));
    let light_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.85, 0.85, 0.85),
        ..default()
    });
    let dark_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.65, 0.65, 0.70),
        ..default()
    });

    for y in 0..GRID_H {
        for x in 0..GRID_W {
            let mat = if (x + y) % 2 == 0 {
                light_mat.clone()
            } else {
                dark_mat.clone()
            };
            commands.spawn((
                Mesh3d(cell_mesh.clone()),
                MeshMaterial3d(mat),
                Transform::from_xyz(x as f32 * CELL_SIZE, 0.0, y as f32 * CELL_SIZE),
            ));
        }
    }

    // Goal marker — glowing golden cylinder.
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

    // Agent cubes.
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
            Transform::from_translation(grid_to_world(pos)),
            AgentMarker { kind },
            Visibility::Visible,
        ));
    }

    // Light.
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            intensity: 3_000_000.0,
            range: 50.0,
            ..default()
        },
        Transform::from_xyz(
            GRID_W as f32 * 0.5,
            12.0,
            GRID_H as f32 * 0.5,
        ),
    ));

    // Camera — isometric-ish view.
    let center = Vec3::new(
        (GRID_W as f32 - 1.0) * CELL_SIZE * 0.5,
        0.0,
        (GRID_H as f32 - 1.0) * CELL_SIZE * 0.5,
    );
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(center.x - 6.0, 10.0, center.z + 10.0)
            .looking_at(center, Vec3::Y),
    ));

    // Simulation state resource.
    let fsm = FSMAgent::with_config(0, 0, 0.15, 10, 0.995);
    let astar = AStarAgent::with_config(0, 0, Some(30), 0.1, 10, 0.995);
    let bt = BehaviorTreeAgent::with_config(0, 0, 0.15, 10, 0.995);

    commands.insert_resource(SimState {
        grid,
        fsm,
        astar,
        bt,
        tick_timer: Timer::from_seconds(TICK_INTERVAL, TimerMode::Repeating),
        total_ticks: 0,
        fsm_done: false,
        astar_done: false,
        bt_done: false,
        all_done_printed: false,
    });
}

// ─── Simulation tick ────────────────────────────────────────
fn tick_simulation(
    time: Res<Time>,
    mut sim: ResMut<SimState>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    sim.tick_timer.tick(time.delta());

    if !sim.tick_timer.just_finished() {
        return;
    }

    // Stop ticking once everyone is done.
    if sim.is_all_done() {
        if !sim.all_done_printed {
            sim.all_done_printed = true;
            println!("=== All agents reached the goal in {} ticks ===", sim.total_ticks);
        }
        return;
    }

    sim.total_ticks += 1;

    // Spawn trail dots for agents that are still active.
    let trail_mesh = meshes.add(Sphere::new(0.08));
    let active_agents: Vec<(Position, AgentKind)> = [
        (!sim.fsm_done, sim.fsm.position(), AgentKind::Fsm),
        (!sim.astar_done, sim.astar.position(), AgentKind::AStar),
        (!sim.bt_done, sim.bt.position(), AgentKind::BehaviorTree),
    ]
    .iter()
    .filter(|(active, _, _)| *active)
    .map(|(_, pos, kind)| (*pos, *kind))
    .collect();

    for (pos, kind) in &active_agents {
        let color = agent_color(*kind).with_alpha(0.3);
        commands.spawn((
            Mesh3d(trail_mesh.clone()),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: color,
                alpha_mode: AlphaMode::Blend,
                ..default()
            })),
            Transform::from_translation(Vec3::new(
                pos.x as f32 * CELL_SIZE,
                0.08,
                pos.y as f32 * CELL_SIZE,
            )),
            TrailDot,
        ));
    }

    // Tick only active agents.
    let grid = sim.grid.clone();

    if !sim.fsm_done {
        sim.fsm.update(&grid);
        if sim.fsm.state() == FSMState::FoundGoal {
            sim.fsm_done = true;
            println!("✓ FSM reached goal at tick {}", sim.total_ticks);
        }
    }

    if !sim.astar_done {
        sim.astar.update(&grid);
        if sim.astar.position() == grid.goal || sim.astar.is_stuck() {
            sim.astar_done = true;
            if sim.astar.position() == grid.goal {
                println!("✓ A* reached goal at tick {}", sim.total_ticks);
            } else {
                println!("✗ A* got stuck at tick {}", sim.total_ticks);
            }
        }
    }

    if !sim.bt_done {
        sim.bt.update(&grid);
        if sim.bt.position() == grid.goal {
            sim.bt_done = true;
            println!("✓ BT reached goal at tick {}", sim.total_ticks);
        }
    }
}

// ─── Sync agent visuals ────────────────────────────────────
fn sync_agents(
    sim: Res<SimState>,
    mut query: Query<(&AgentMarker, &mut Transform, &mut Visibility)>,
) {
    for (marker, mut transform, mut visibility) in &mut query {
        let done = match marker.kind {
            AgentKind::Fsm => sim.fsm_done,
            AgentKind::AStar => sim.astar_done,
            AgentKind::BehaviorTree => sim.bt_done,
        };

        if done {
            // Hide the cube once the agent finishes.
            *visibility = Visibility::Hidden;
            continue;
        }

        let pos = match marker.kind {
            AgentKind::Fsm => sim.fsm.position(),
            AgentKind::AStar => sim.astar.position(),
            AgentKind::BehaviorTree => sim.bt.position(),
        };

        let target = grid_to_world(pos);
        // Smooth lerp for fluid movement.
        transform.translation = transform.translation.lerp(target, 0.15);
    }
}

// ─── Rotate goal marker ────────────────────────────────────
fn rotate_goal(
    time: Res<Time>,
    mut query: Query<&mut Transform, With<GoalMarker>>,
) {
    for mut transform in &mut query {
        transform.rotate_y(time.delta_secs() * 1.5);
    }
}
