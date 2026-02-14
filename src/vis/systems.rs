use bevy::prelude::*;
use bevy::math::Isometry3d;
use rand::Rng;
use crate::engine::world::Position;
use crate::agents::fsm::FSMState;
use crate::agents::Agent;

use super::components::{AgentKind, AgentMarker, TrailDot, Shaking, Obstacle, GoalMarker};
use super::resources::{SimState, UiState, HeatmapMaterials};

// Constants moved here or imported? We can define local constants for simplicity in this refactor.
const CELL_SIZE: f32 = 1.0;
const BASE_TICK_INTERVAL: f32 = 0.25;
const AGENT_Y: f32 = 0.35;
const GRID_W: usize = 12;
const GRID_H: usize = 8;
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

pub fn tick_simulation(
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

    sim.tick_timer += time.delta_secs() * ui_state.time_scale;

    while sim.tick_timer >= BASE_TICK_INTERVAL {
        sim.tick_timer -= BASE_TICK_INTERVAL;
        
        if sim.is_all_done() {
            if !sim.all_done_printed {
                sim.all_done_printed = true;
                println!("=== All agents reached the goal in {} ticks ===", sim.total_ticks);
            }
            continue;
        }

        sim.total_ticks += 1;

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

pub fn render_heatmap(
    sim: Res<SimState>,
    ui_state: Res<UiState>,
    heatmap_mats: Res<HeatmapMaterials>,
    mut query: Query<&mut MeshMaterial3d<StandardMaterial>>,
) {
    // FIX 9.1: We run this loop ALWAYS, to actively revert colors if toggled off
    for y in 0..GRID_H {
        for x in 0..GRID_W {
            let entity = sim.grid_tile_entities[y][x];
            if entity == Entity::PLACEHOLDER { continue; }

            if let Ok(mut mat) = query.get_mut(entity) {
                let default_mat = if (x + y) % 2 == 0 { 
                    heatmap_mats.default_light.clone() 
                } else { 
                    heatmap_mats.default_dark.clone() 
                };

                // If disabled, we WANT default mat.
                let desired_mat = if !ui_state.show_heatmap {
                     default_mat
                } else {
                    // If enabled, check visitors
                    let visitors = sim.cell_visitors.get(&(x, y));
                    if let Some(visitors) = visitors {
                        if visitors.len() > 1 {
                            heatmap_mats.multi_visited.clone()
                        } else if visitors.contains(&AgentKind::Fsm) {
                            heatmap_mats.fsm_visited.clone()
                        } else if visitors.contains(&AgentKind::AStar) {
                            heatmap_mats.astar_visited.clone()
                        } else if visitors.contains(&AgentKind::BehaviorTree) {
                            heatmap_mats.bt_visited.clone()
                        } else {
                            default_mat
                        }
                    } else {
                        default_mat
                    }
                };

                if mat.0 != desired_mat {
                    mat.0 = desired_mat;
                }
            }
        }
    }
}

pub fn handle_visual_events(
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
            commands.entity(entity).insert(Shaking {
                timer: Timer::from_seconds(0.4, TimerMode::Once),
                magnitude: 0.15,
            });
        }
    }
}

pub fn apply_shake(
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
            let offset = Vec3::new(
                rng.gen_range(-shaking.magnitude..shaking.magnitude),
                rng.gen_range(0.0..shaking.magnitude),
                rng.gen_range(-shaking.magnitude..shaking.magnitude),
            );
            transform.translation += offset;
        }
    }
}

pub fn sync_agents(
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
        transform.translation = transform.translation.lerp(target, 0.15);
    }
}

pub fn draw_gizmos(
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

pub fn rotate_goal(
    time: Res<Time>,
    mut query: Query<&mut Transform, With<GoalMarker>>,
) {
    for mut transform in &mut query {
        transform.rotate_y(time.delta_secs() * 1.5);
    }
}

pub fn render_obstacles(
    mut commands: Commands,
    sim: Res<SimState>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    obstacle_query: Query<Entity, With<Obstacle>>,
) {
    if sim.total_ticks == 0 {
        let grid_obstacles = sim.grid.obstacle_positions();
        let current_count = obstacle_query.iter().count();

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
