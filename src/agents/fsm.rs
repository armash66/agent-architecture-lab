use rand::Rng;
use rand::seq::SliceRandom;

use crate::engine::world::{Grid, Position};
use super::memory::SpatialMemory;

/// FSM states for the agent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FSMState {
    Exploring,
    Resting,
    FoundGoal,
}

/// Simple action enum produced by the decision function.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    MoveRandomly,
    Rest,
    None,
}

/// FSM-based agent with optional cognitive limitations.
pub struct FSMAgent {
    pos: Position,
    state: FSMState,
    energy: u32,
    /// Base noise probability (0.0–1.0).
    noise: f32,
    /// Current exploration multiplier (starts at 1.0, decays each tick).
    exploration_rate: f32,
    /// Per-tick multiplicative decay for exploration_rate (e.g. 0.995).
    decay_rate: f32,
    /// Visited-cell memory with bounded capacity.
    memory: SpatialMemory,
}

impl FSMAgent {
    /// Create a new FSM agent at the given starting coordinates.
    pub fn new(start_x: usize, start_y: usize) -> Self {
        Self {
            pos: Position {
                x: start_x,
                y: start_y,
            },
            state: FSMState::Exploring,
            energy: 100,
            noise: 0.0,
            exploration_rate: 1.0,
            decay_rate: 1.0,
            memory: SpatialMemory::new(0),
        }
    }

    /// Create an FSM agent with decision noise.
    pub fn with_noise(start_x: usize, start_y: usize, noise: f32) -> Self {
        Self {
            noise,
            ..Self::new(start_x, start_y)
        }
    }

    /// Create an FSM agent with full cognitive config.
    pub fn with_config(
        start_x: usize,
        start_y: usize,
        noise: f32,
        memory_capacity: usize,
        decay_rate: f32,
    ) -> Self {
        Self {
            noise,
            decay_rate,
            memory: SpatialMemory::new(memory_capacity),
            ..Self::new(start_x, start_y)
        }
    }

    /// Expose read-only state for the world/printing.
    pub fn state(&self) -> FSMState {
        self.state
    }

    pub fn energy(&self) -> u32 {
        self.energy
    }

    pub fn position(&self) -> Position {
        self.pos
    }

    /// Decide the next high-level action based on current state,
    /// internal energy, and environment.
    pub fn decide_next_action(&self, grid: &Grid) -> Action {
        // If we've already reached the goal, no further actions.
        if self.state == FSMState::FoundGoal || self.pos == grid.goal {
            return Action::None;
        }

        match self.state {
            FSMState::Exploring => {
                if self.energy < 10 {
                    Action::Rest
                } else {
                    Action::MoveRandomly
                }
            }
            FSMState::Resting => {
                // While resting we don't move; once energy is high enough
                // the state machine will switch back to Exploring.
                Action::Rest
            }
            FSMState::FoundGoal => Action::None,
        }
    }

    /// Update the FSM: handle transitions, perform actions,
    /// and print state changes.
    pub fn update(&mut self, grid: &Grid) {
        // Record current position in memory.
        self.memory.record(self.pos);

        // Decay exploration rate.
        self.exploration_rate *= self.decay_rate;
        // Check for goal condition first.
        if self.pos == grid.goal && self.state != FSMState::FoundGoal {
            self.state = FSMState::FoundGoal;
            println!(
                "FSM: Reached goal at ({}, {}) -> state = FoundGoal",
                self.pos.x, self.pos.y
            );
            return;
        }

        // State transitions based on energy.
        match self.state {
            FSMState::Exploring if self.energy < 10 => {
                self.state = FSMState::Resting;
                println!(
                    "FSM: Energy low ({}). Transition Exploring -> Resting",
                    self.energy
                );
            }
            FSMState::Resting if self.energy >= 100 => {
                self.state = FSMState::Exploring;
                println!(
                    "FSM: Energy full ({}). Transition Resting -> Exploring",
                    self.energy
                );
            }
            _ => {}
        }

        let action = self.decide_next_action(grid);

        // Decision noise (modulated by exploration rate).
        let effective_noise = self.noise * self.exploration_rate;
        let mut rng = rand::thread_rng();
        if effective_noise > 0.0 && rng.r#gen::<f32>() < effective_noise {
            if let Some((nx, ny)) = grid.random_walkable_neighbor(self.pos.x, self.pos.y) {
                self.pos = Position { x: nx, y: ny };
                if self.energy > 0 {
                    self.energy -= 1;
                }
                println!("FSM: Noise! Random move to ({}, {})", nx, ny);
                return;
            }
        }

        match action {
            Action::MoveRandomly => {
                println!(
                    "FSM: Exploring at ({}, {}), energy = {}. Moving...",
                    self.pos.x, self.pos.y, self.energy
                );
                self.move_randomly(grid);
                // Exploring costs a bit of energy.
                if self.energy > 0 {
                    self.energy -= 1;
                }
            }
            Action::Rest => {
                // Resting recovers energy.
                let before = self.energy;
                self.energy = (self.energy + 10).min(100);
                println!(
                    "FSM: Resting at ({}, {}), energy {} -> {}",
                    self.pos.x, self.pos.y, before, self.energy
                );
            }
            Action::None => {
                // Do nothing (e.g., FoundGoal).
            }
        }
    }

    fn move_randomly(&mut self, grid: &Grid) {
        let mut rng = rand::thread_rng();

        // Collect all valid neighbors.
        let mut candidates = Vec::new();
        if self.pos.x > 0 && grid.is_walkable(self.pos.x - 1, self.pos.y) {
            candidates.push(Position { x: self.pos.x - 1, y: self.pos.y });
        }
        if self.pos.x + 1 < grid.width && grid.is_walkable(self.pos.x + 1, self.pos.y) {
            candidates.push(Position { x: self.pos.x + 1, y: self.pos.y });
        }
        if self.pos.y > 0 && grid.is_walkable(self.pos.x, self.pos.y - 1) {
            candidates.push(Position { x: self.pos.x, y: self.pos.y - 1 });
        }
        if self.pos.y + 1 < grid.height && grid.is_walkable(self.pos.x, self.pos.y + 1) {
            candidates.push(Position { x: self.pos.x, y: self.pos.y + 1 });
        }

        if candidates.is_empty() {
            return;
        }

        // Prefer unvisited cells if memory is active.
        let unvisited: Vec<_> = candidates.iter().filter(|p| !self.memory.contains(p)).copied().collect();
        let pool = if unvisited.is_empty() { &candidates } else { &unvisited };

        if let Some(&next) = pool.choose(&mut rng) {
            self.pos = next;
        }
    }
}

impl super::Agent for FSMAgent {
    fn update(&mut self, grid: &Grid) {
        self.update(grid);
    }

    fn position(&self) -> Position {
        self.pos
    }

    fn name(&self) -> &'static str {
        "FSM"
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn energy(&self) -> Option<u32> {
        Some(self.energy)
    }

    fn debug_state(&self) -> String {
        format!("{:?}", self.state)
    }
}

