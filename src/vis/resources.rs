use bevy::prelude::*;
use std::collections::{HashMap, HashSet};
use crate::engine::world::{Grid, Position};
use crate::agents::fsm::FSMAgent;
use crate::agents::astar::AStarAgent;
use crate::agents::behavior_tree::BehaviorTreeAgent;
use super::components::AgentKind;

#[derive(Resource)]
pub struct UiState {
    pub paused: bool,
    pub time_scale: f32,
    pub show_heatmap: bool,
    pub show_path_gizmos: bool,
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
pub struct SimState {
    pub grid: Grid,
    pub fsm: FSMAgent,
    pub astar: AStarAgent,
    pub bt: BehaviorTreeAgent,
    pub tick_timer: f32,
    pub total_ticks: u32,
    pub fsm_done: bool,
    pub astar_done: bool,
    pub bt_done: bool,
    pub all_done_printed: bool,
    pub cell_visitors: HashMap<(usize, usize), HashSet<AgentKind>>,
    pub grid_tile_entities: Vec<Vec<Entity>>,
}

impl SimState {
    pub fn is_all_done(&self) -> bool {
        self.fsm_done && self.astar_done && self.bt_done
    }

    pub fn update_visits(&mut self, pos: Position, kind: AgentKind) {
        if self.grid.is_walkable(pos.x, pos.y) {
            self.cell_visitors
                .entry((pos.x, pos.y))
                .or_default()
                .insert(kind);
        }
    }
    
    pub fn reset(&mut self, mut grid: Grid, obstacle_density: f32) {
        grid.scatter_obstacles(obstacle_density);
        
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
    }
}

#[derive(Resource)]
pub struct HeatmapMaterials {
    pub default_light: Handle<StandardMaterial>,
    pub default_dark: Handle<StandardMaterial>,
    pub fsm_visited: Handle<StandardMaterial>,
    pub astar_visited: Handle<StandardMaterial>,
    pub bt_visited: Handle<StandardMaterial>,
    pub multi_visited: Handle<StandardMaterial>,
}
