pub mod app;
pub mod camera;
pub mod components;
pub mod resources;
pub mod systems;
pub mod ui;

use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use resources::UiState;

pub fn run() {
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
        .add_systems(Startup, app::setup)
        .add_systems(Update, (
            camera::orbit_camera,
            ui::ui_system,
            systems::tick_simulation,
            systems::sync_agents,
            systems::handle_visual_events,
            systems::apply_shake,
            systems::render_heatmap,
            systems::render_obstacles,
            systems::rotate_goal,
            systems::draw_gizmos,
        ))
        .run();
}
