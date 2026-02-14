use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use crate::engine::world::{Grid, Position};
use super::resources::{SimState, UiState};
use super::components::{TrailDot, Obstacle};

pub fn ui_system(
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
                let w = sim.grid.width;
                let h = sim.grid.height;
                let grid = Grid::new(w, h, Position { x: w-1, y: h-1 });
                // We don't have access to OBSTACLE_DENSITY constant here easily unless we move it or duplicate
                // For now, hardcode or access from existing config if available.
                // Or let's just use 0.15 matching viewer.rs constant.
                sim.reset(grid, 0.15);
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
