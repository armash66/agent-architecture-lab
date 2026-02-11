use rand::Rng;

use crate::engine::world::{Grid, Position};

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

/// Deterministic FSM-based agent.
pub struct FSMAgent {
    pos: Position,
    state: FSMState,
    energy: u32,
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

        // Try random directions until we find a valid in-bounds move.
        loop {
            let dir = rng.gen_range(0..4);
            let mut new_pos = self.pos;

            match dir {
                // left
                0 if new_pos.x > 0 => new_pos.x -= 1,
                // right
                1 if new_pos.x + 1 < grid.width => new_pos.x += 1,
                // up
                2 if new_pos.y > 0 => new_pos.y -= 1,
                // down
                3 if new_pos.y + 1 < grid.height => new_pos.y += 1,
                _ => {
                    // invalid move (would go out of bounds), try again
                    continue;
                }
            }

            self.pos = new_pos;
            break;
        }
    }
}

