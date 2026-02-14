//! 3D viewer for the Cognitive Grid simulation using Bevy 0.15.
//!
//! Run with: `cargo run --bin viewer`
//!
//! Controls:
//!   - Left mouse drag: orbit camera
//!   - Scroll wheel: zoom in/out
//!   - UI Panel: Speed slider, Pause/Resume, Restart, Visual toggles

use bevy::prelude::*;
use bevy::input::mouse::{MouseMotion, MouseScrollUnit, MouseWheel};
use bevy::math::Isometry3d;
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use std::collections::{HashMap, HashSet};
use rand::Rng;

use cognitive_grid::agents::fsm::{FSMAgent, FSMState};
use cognitive_grid::agents::astar::AStarAgent;
use cognitive_grid::agents::behavior_tree::BehaviorTreeAgent;
use cognitive_grid::engine::world::{Grid, Position};
// Trait must be imported to use methods like did_noise_trigger()
use cognitive_grid::agents::Agent; 

// ─── Constants ──────────────────────────────────────────────
const GRID_W: usize = 12;
const GRID_H: usize = 8;
const CELL_SIZE: f32 = 1.0;
const AGENT_Y: f32 = 0.35;
const BASE_TICK_INTERVAL: f32 = 0.25;
const OBSTACLE_DENSITY: f32 = 0.15;

// ─── Components / Resources ────────────────────────────────
#[derive(Component)]
struct AgentMarker {
    kind: AgentKind,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
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
struct OrbitCamera {
    focus: Vec3,
    radius: f32,
    yaw: f32,   // radians
    pitch: f32, // radians
}

#[derive(Component)]
struct Shaking {
    timer: Timer,
    magnitude: f32,
}

#[derive(Resource)]
struct UiState {
    paused: bool,
    time_scale: f32,
    show_heatmap: bool,
    show_path_gizmos: bool,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            paused: false,
            time_scale: 1.0,
            show_heatmap: true,
            show_path_gizmos: true,
        }
    }
}

#[derive(Resource)]
struct SimState {
    grid: Grid,
    fsm: FSMAgent,
    astar: AStarAgent,
    bt: BehaviorTreeAgent,
    // Accumulator for tick timing
    tick_timer: f32,
    total_ticks: u32,
    fsm_done: bool,
    astar_done: bool,
    bt_done: bool,
    all_done_printed: bool,
    // Tracks which agents visited a cell: (x, y) -> Set of AgentKinds
    cell_visitors: HashMap<(usize, usize), HashSet<AgentKind>>,
    grid_tile_entities: Vec<Vec<Entity>>,
}

#[derive(Resource)]
struct HeatmapMaterials {
    default_light: Handle<StandardMaterial>,
    default_dark: Handle<StandardMaterial>,
    // Agent-specific colors
    fsm_visited: Handle<StandardMaterial>,   // Greenish
    astar_visited: Handle<StandardMaterial>, // Blueish
    bt_visited: Handle<StandardMaterial>,    // Orangeish
    multi_visited: Handle<StandardMaterial>, // Mixed / Grey
}

impl SimState {
    fn is_all_done(&self) -> bool {
        self.fsm_done && self.astar_done && self.bt_done
    }

    fn update_visits(&mut self, pos: Position, kind: AgentKind) {
        if self.grid.is_walkable(pos.x, pos.y) {
            self.cell_visitors
                .entry((pos.x, pos.y))
                .or_default()
                .insert(kind);
        }
    }
    
    // Reset simulation state for restart
    fn reset(&mut self, mut grid: Grid) {
        grid.scatter_obstacles(OBSTACLE_DENSITY);
        
        self.grid = grid;
        self.fsm = FSMAgent::with_config(0, 0, 0.15, 10, 0.995);
        self.astar = AStarAgent::with_config(0, 0, Some(30), 0.1, 10, 0.995);
        self.bt = BehaviorTreeAgent::with_config(0, 0, 0.15, 10, 0.995);
        self.tick_timer = 0.0;
        self.total_ticks = 0;
        self.fsm_done = false;
        self.astar_done = false;
        self.bt_done = false;
        self.all_done_printed = false;
        self.cell_visitors.clear();
        self.cell_visitors.clear();
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
        .add_plugins(EguiPlugin)
        .init_resource::<UiState>()
        .add_systems(Startup, setup)
        .add_systems(Update, (
            orbit_camera,
            ui_system,
            tick_simulation,
            sync_agents,
            handle_visual_events,
            apply_shake,
            render_heatmap,
            render_obstacles, // New system to update obstacles on restart
            rotate_goal,
            draw_gizmos,
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

    // ── Materials ──────────────────────────
    let default_light = materials.add(StandardMaterial {
        base_color: Color::srgb(0.85, 0.85, 0.85),
        ..default()
    });
    let default_dark = materials.add(StandardMaterial {
        base_color: Color::srgb(0.65, 0.65, 0.70),
        ..default()
    });

    // Agent-specific trail colors (slightly distinct from agent cubes for tiles)
    let fsm_visited = materials.add(StandardMaterial {
        base_color: Color::srgb(0.6, 0.9, 0.7), // light green
        ..default()
    });
    let astar_visited = materials.add(StandardMaterial {
        base_color: Color::srgb(0.6, 0.7, 0.95), // light blue
        ..default()
    });
    let bt_visited = materials.add(StandardMaterial {
        base_color: Color::srgb(0.95, 0.7, 0.6), // light orange
        ..default()
    });
    let multi_visited = materials.add(StandardMaterial {
        base_color: Color::srgb(0.6, 0.5, 0.6), // purple/grey mix
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
            // Spawn tile regardless of obstacle status
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

// ─── UI System ──────────────────────────────────────────────
fn ui_system(
    mut contexts: EguiContexts,
    mut ui_state: ResMut<UiState>,
    mut sim: ResMut<SimState>,
    mut commands: Commands,
    obstacle_query: Query<Entity, With<Obstacle>>,
    trail_query: Query<Entity, With<TrailDot>>,
) {
    let ctx = contexts.ctx_mut();

    egui::SidePanel::left("control_panel")
        .default_width(220.0)
        .show(ctx, |ui| {
            ui.heading("Cognitive Grid");
            ui.separator();

            ui.horizontal(|ui| {
                ui.label(format!("Tick: {}", sim.total_ticks));
            });
            ui.separator();

            // Speed Control
            ui.heading("Controls");
            if ui_state.paused {
                if ui.button("▶ Resume").clicked() {
                    ui_state.paused = false;
                }
            } else {
                if ui.button("⏸ Pause").clicked() {
                    ui_state.paused = true;
                }
            }
            
            ui.add(egui::Slider::new(&mut ui_state.time_scale, 0.1..=50.0).text("Speed"));
            if ui.button("1x").clicked() { ui_state.time_scale = 1.0; }
            if ui.button("10x").clicked() { ui_state.time_scale = 10.0; }
            if ui.button("50x (Hyper)").clicked() { ui_state.time_scale = 50.0; }

            ui.separator();

            // Restart
            if ui.button("🔄 Restart Simulation").clicked() {
                // Despawn trails
                for entity in &trail_query {
                    commands.entity(entity).despawn_recursive();
                }
                
                // Reset sim state
                let grid = Grid::new(GRID_W, GRID_H, Position { x: GRID_W-1, y: GRID_H-1 });
                sim.reset(grid);
            }

            ui.separator();

            // Toggles
            ui.heading("Visuals");
            ui.checkbox(&mut ui_state.show_heatmap, "Show Heatmap");
            ui.checkbox(&mut ui_state.show_path_gizmos, "Show Planning Radius");

            ui.separator();

            // Agent Status
            ui.heading("Agents");
            let status = |done: bool, pos: Position| -> String {
                if done { "Done ✓".to_string() } else { format!("({}, {})", pos.x, pos.y) }
            };
            
            ui.label(format!("FSM: {}", status(sim.fsm_done, sim.fsm.position())));
            ui.label(format!("A*: {}", status(sim.astar_done, sim.astar.position())));
            ui.label(format!("BT: {}", status(sim.bt_done, sim.bt.position())));
        });
}

// ─── Obstacle Handling ─────────────────────────────────────
#[derive(Component)]
struct Obstacle;

fn render_obstacles(
    mut commands: Commands,
    sim: Res<SimState>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    obstacle_query: Query<Entity, With<Obstacle>>,
) {
    // Only update on fresh start or reset
    if sim.total_ticks == 0 {
        // Only despawn if they exist (to avoid flicker every frame of tick 0)
        // Check if count mismatches or logic flag?
        // Simple hack: We cleared sim.grid on reset, so we rebuild.
        // But render_obstacles runs every frame. We need a way to detect "just reset".
        // Let's rely on checking if obstacles exist vs grid count.
        
        let grid_obstacles = sim.grid.obstacle_positions();
        let current_count = obstacle_query.iter().count();

        // If mismatched, rebuild.
        // (Note: this might rebuild once at startup which is fine)
        if current_count != grid_obstacles.len() {
            for entity in &obstacle_query {
                commands.entity(entity).despawn();
            }

            let obstacle_mesh = meshes.add(Cuboid::new(CELL_SIZE * 0.95, 0.5, CELL_SIZE * 0.95));
            let obstacle_mat = materials.add(StandardMaterial {
                base_color: Color::srgb(0.6, 0.15, 0.15),
                ..default()
            });

            for (x, y) in grid_obstacles {
                commands.spawn((
                    Mesh3d(obstacle_mesh.clone()),
                    MeshMaterial3d(obstacle_mat.clone()),
                    Transform::from_xyz(x as f32 * CELL_SIZE, 0.25, y as f32 * CELL_SIZE),
                    Obstacle,
                ));
            }
        }
    }
}

// ─── Orbital camera ─────────────────────────────────────────
fn orbit_camera(
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut mouse_motion: EventReader<MouseMotion>,
    mut scroll_events: EventReader<MouseWheel>,
    mut query: Query<(&mut OrbitCamera, &mut Transform)>,
    mut contexts: EguiContexts,
) {
    // Don't move camera if interacting with UI
    if contexts.ctx_mut().is_pointer_over_area() {
        return;
    }

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
    ui_state: Res<UiState>,
    mut sim: ResMut<SimState>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if ui_state.paused {
        return;
    }

    // Accumulate time scaled by speed
    sim.tick_timer += time.delta_secs() * ui_state.time_scale;

    // Run as many ticks as fit in the accumulator
    while sim.tick_timer >= BASE_TICK_INTERVAL {
        sim.tick_timer -= BASE_TICK_INTERVAL;
        
        if sim.is_all_done() {
            if !sim.all_done_printed {
                sim.all_done_printed = true;
                println!("=== All agents reached the goal in {} ticks ===", sim.total_ticks);
            }
            continue; // Keep running loop to drain timer, but don't update agents
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
            let pos = sim.fsm.position();
            sim.update_visits(pos, AgentKind::Fsm);
            if sim.fsm.state() == FSMState::FoundGoal {
                sim.fsm_done = true;
                println!("✓ FSM reached goal at tick {}", sim.total_ticks);
            }
        }

        if !sim.astar_done {
            sim.astar.update(&grid);
             let pos = sim.astar.position();
            sim.update_visits(pos, AgentKind::AStar);
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
             let pos = sim.bt.position();
            sim.update_visits(pos, AgentKind::BehaviorTree);
            if sim.bt.position() == grid.goal {
                sim.bt_done = true;
                println!("✓ BT reached goal at tick {}", sim.total_ticks);
            }
        }
    }
}

// ─── Visual Features (Polish) ──────────────────────────────

fn handle_visual_events(
    mut commands: Commands,
    sim: Res<SimState>,
    query: Query<(Entity, &AgentMarker)>,
) {
    for (entity, marker) in &query {
        let triggered = match marker.kind {
            AgentKind::Fsm => sim.fsm.did_noise_trigger(),
            AgentKind::AStar => sim.astar.did_noise_trigger(),
            AgentKind::BehaviorTree => sim.bt.did_noise_trigger(),
        };

        if triggered {
            // Start shaking
            commands.entity(entity).insert(Shaking {
                timer: Timer::from_seconds(0.4, TimerMode::Once),
                magnitude: 0.15,
            });
        }
    }
}

fn apply_shake(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut Shaking)>,
) {
    let mut rng = rand::thread_rng();

    for (entity, mut transform, mut shaking) in &mut query {
        shaking.timer.tick(time.delta());

        if shaking.timer.finished() {
            commands.entity(entity).remove::<Shaking>();
        } else {
            // Jiggle
            let offset = Vec3::new(
                rng.gen_range(-shaking.magnitude..shaking.magnitude),
                rng.gen_range(0.0..shaking.magnitude),
                rng.gen_range(-shaking.magnitude..shaking.magnitude),
            );
            transform.translation += offset;
        }
    }
}

fn render_heatmap(
    sim: Res<SimState>,
    ui_state: Res<UiState>,
    heatmap_mats: Res<HeatmapMaterials>,
    mut query: Query<&mut MeshMaterial3d<StandardMaterial>>,
) {
    if !ui_state.show_heatmap {
        return;
    }

    for y in 0..GRID_H {
        for x in 0..GRID_W {
            let entity = sim.grid_tile_entities[y][x];
            if entity == Entity::PLACEHOLDER { continue; }

            if let Ok(mut mat) = query.get_mut(entity) {
                let visitors = sim.cell_visitors.get(&(x, y));
                
                let desired_mat = if let Some(visitors) = visitors {
                    if visitors.len() > 1 {
                        heatmap_mats.multi_visited.clone()
                    } else if visitors.contains(&AgentKind::Fsm) {
                        heatmap_mats.fsm_visited.clone()
                    } else if visitors.contains(&AgentKind::AStar) {
                        heatmap_mats.astar_visited.clone()
                    } else if visitors.contains(&AgentKind::BehaviorTree) {
                        heatmap_mats.bt_visited.clone()
                    } else {
                        if (x + y) % 2 == 0 { heatmap_mats.default_light.clone() } else { heatmap_mats.default_dark.clone() }
                    }
                } else {
                    if (x + y) % 2 == 0 { heatmap_mats.default_light.clone() } else { heatmap_mats.default_dark.clone() }
                };

                if mat.0 != desired_mat {
                    mat.0 = desired_mat;
                }
            }
        }
    }
}

fn draw_gizmos(
    mut gizmos: Gizmos,
    sim: Res<SimState>,
    ui_state: Res<UiState>,
    query: Query<(&AgentMarker, &Transform, &Visibility)>,
) {
    if !ui_state.show_path_gizmos {
        return;
    }

    for (marker, transform, vis) in &query {
        if vis == Visibility::Hidden { continue; }

        let radius = match marker.kind {
            AgentKind::Fsm => sim.fsm.planning_radius(),
            AgentKind::AStar => sim.astar.planning_radius(),
            AgentKind::BehaviorTree => sim.bt.planning_radius(),
        };

        if let Some(r) = radius {
            let rotation = Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2);
            let isometry = Isometry3d::new(transform.translation, rotation);
            
            gizmos.circle(
                isometry,
                r,
                Color::srgb(0.3, 0.5, 1.0).with_alpha(0.5),
            );
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

        *visibility = Visibility::Visible;

        let pos = match marker.kind {
            AgentKind::Fsm => sim.fsm.position(),
            AgentKind::AStar => sim.astar.position(),
            AgentKind::BehaviorTree => sim.bt.position(),
        };

        let target = grid_to_world(pos, agent_y_offset(marker.kind));
        // We use lerp for smooth aesthetic, but Shaking system will add noise on top
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
