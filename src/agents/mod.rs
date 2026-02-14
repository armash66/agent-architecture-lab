use crate::engine::world::{Grid, Position};

pub trait Agent {
    fn update(&mut self, grid: &Grid);
    fn position(&self) -> Position;
    fn name(&self) -> &'static str;
    fn as_any(&self) -> &dyn std::any::Any; // Helpful for downcasting if needed
    fn is_stuck(&self) -> bool { false }
    fn energy(&self) -> Option<u32> { None }
    fn debug_state(&self) -> String { String::new() }
    
    // Visual feedback hooks
    fn did_noise_trigger(&self) -> bool { false }
    fn planning_radius(&self) -> Option<f32> { None }
}

pub mod fsm;
pub mod astar;
pub mod behavior_tree;
pub mod memory;

