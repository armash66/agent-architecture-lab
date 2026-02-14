//! 3D viewer for the Cognitive Grid simulation using Bevy 0.15.
//!
//! Run with: `cargo run --bin viewer`
//!
//! Controls:
//!   - Left mouse drag: orbit camera
//!   - Scroll wheel: zoom in/out

use bevy::prelude::*;
use bevy::input::mouse::{MouseMotion, MouseScrollUnit, MouseWheel};

use cognitive_grid::agents::fsm::{FSMAgent, FSMState};
use cognitive_grid::agents::astar::AStarAgent;
use cognitive_grid::agents::behavior_tree::BehaviorTreeAgent;
use cognitive_grid::engine::world::{Grid, Position};

// ─── Constants ──────────────────────────────────────────────
const GRID_W: usize = 12;
const GRID_H: usize = 8;
const CELL_SIZE: f32 = 1.0;
const AGENT_Y: f32 = 0.35;
const TICK_INTERVAL: f32 = 0.25;
const OBSTACLE_DENSITY: f32 = 0.15;

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

#[derive(Component)]
struct HudText;

#[derive(Component)]
struct OrbitCamera {
    focus: Vec3,
    radius: f32,
    yaw: f32,   // radians
    pitch: f32, // radians
}

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

fn agent_label(kind: AgentKind) -> &'static str {
    match kind {
        AgentKind::Fsm => "FSM",
        AgentKind::AStar => "A*",
        AgentKind::BehaviorTree => "BT",
    }
}

// Vertical offsets to prevent z-fighting when agents overlap.
fn agent_y_offset(kind: AgentKind) -> f32 {
    match kind {
        AgentKind::Fsm => AGENT_Y,
        AgentKind::AStar => AGENT_Y + 0.01,
        AgentKind::BehaviorTree => AGENT_Y + 0.02,
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
        .add_systems(Update, (
            orbit_camera,
            tick_simulation,
            sync_agents,
            rotate_goal,
            update_hud,
        ))
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
    let mut grid = Grid::new(GRID_W, GRID_H, goal);
    grid.scatter_obstacles(OBSTACLE_DENSITY);

    // ── Ground plane ────────────────────────────────────
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
            if !grid.is_walkable(x, y) {
                continue; // obstacles get their own mesh
            }
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

    // ── Obstacle cubes ──────────────────────────────────
    let obstacle_mesh = meshes.add(Cuboid::new(CELL_SIZE * 0.95, 0.5, CELL_SIZE * 0.95));
    let obstacle_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.6, 0.15, 0.15),
        ..default()
    });
    for (x, y) in grid.obstacle_positions() {
        commands.spawn((
            Mesh3d(obstacle_mesh.clone()),
            MeshMaterial3d(obstacle_mat.clone()),
            Transform::from_xyz(x as f32 * CELL_SIZE, 0.25, y as f32 * CELL_SIZE),
        ));
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

    // ── HUD overlay ─────────────────────────────────────
    commands.spawn((
        Text::new("Tick: 0"),
        TextFont {
            font_size: 20.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
        HudText,
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
        tick_timer: Timer::from_seconds(TICK_INTERVAL, TimerMode::Repeating),
        total_ticks: 0,
        fsm_done: false,
        astar_done: false,
        bt_done: false,
        all_done_printed: false,
    });
}

// ─── Orbital camera ─────────────────────────────────────────
fn orbit_camera(
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut mouse_motion: EventReader<MouseMotion>,
    mut scroll_events: EventReader<MouseWheel>,
    mut query: Query<(&mut OrbitCamera, &mut Transform)>,
) {
    let mut rotation_delta = Vec2::ZERO;
    let mut zoom_delta: f32 = 0.0;

    if mouse_button.pressed(MouseButton::Left) {
        for ev in mouse_motion.read() {
            rotation_delta += ev.delta;
        }
    } else {
        mouse_motion.clear();
    }

    for ev in scroll_events.read() {
        match ev.unit {
            MouseScrollUnit::Line => zoom_delta -= ev.y * 1.0,
            MouseScrollUnit::Pixel => zoom_delta -= ev.y * 0.05,
        }
    }

    for (mut orbit, mut transform) in &mut query {
        orbit.yaw -= rotation_delta.x * 0.005;
        orbit.pitch = (orbit.pitch + rotation_delta.y * 0.005).clamp(0.15, 1.4);
        orbit.radius = (orbit.radius + zoom_delta).clamp(5.0, 30.0);

        let cam_pos = orbit.focus + Vec3::new(
            orbit.radius * orbit.pitch.cos() * orbit.yaw.sin(),
            orbit.radius * orbit.pitch.sin(),
            orbit.radius * orbit.pitch.cos() * orbit.yaw.cos(),
        );
        *transform = Transform::from_translation(cam_pos).looking_at(orbit.focus, Vec3::Y);
    }
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

    if sim.is_all_done() {
        if !sim.all_done_printed {
            sim.all_done_printed = true;
            println!("=== All agents reached the goal in {} ticks ===", sim.total_ticks);
        }
        return;
    }

    sim.total_ticks += 1;

    // Spawn trail dots for active agents.
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

    // Tick active agents.
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
            *visibility = Visibility::Hidden;
            continue;
        }

        let pos = match marker.kind {
            AgentKind::Fsm => sim.fsm.position(),
            AgentKind::AStar => sim.astar.position(),
            AgentKind::BehaviorTree => sim.bt.position(),
        };

        let target = grid_to_world(pos, agent_y_offset(marker.kind));
        transform.translation = transform.translation.lerp(target, 0.15);
    }
}

// ─── Goal rotation ──────────────────────────────────────────
fn rotate_goal(
    time: Res<Time>,
    mut query: Query<&mut Transform, With<GoalMarker>>,
) {
    for mut transform in &mut query {
        transform.rotate_y(time.delta_secs() * 1.5);
    }
}

// ─── HUD update ─────────────────────────────────────────────
fn update_hud(
    sim: Res<SimState>,
    mut query: Query<&mut Text, With<HudText>>,
) {
    let status = |done: bool, pos: Position, label: &str| -> String {
        if done {
            format!("{}: ✓ Done", label)
        } else {
            format!("{}: ({},{})", label, pos.x, pos.y)
        }
    };

    let hud = format!(
        "Tick: {}\n{}\n{}\n{}",
        sim.total_ticks,
        status(sim.fsm_done, sim.fsm.position(), "FSM"),
        status(sim.astar_done, sim.astar.position(), "A*"),
        status(sim.bt_done, sim.bt.position(), "BT"),
    );

    for mut text in &mut query {
        **text = hud.clone();
    }
}
