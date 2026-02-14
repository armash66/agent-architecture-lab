use crate::agents::Agent;

pub use super::grid::Grid;
pub use super::position::Position;

/// A world that holds multiple agents navigating the same grid.
pub struct MultiWorld {
    pub grid: Grid,
    pub agents: Vec<Box<dyn Agent>>,
    pub step: usize,
}

impl MultiWorld {
    /// Create a multi-agent world from a pre-built grid and a list of agents.
    pub fn new(grid: Grid, agents: Vec<Box<dyn Agent>>) -> Self {
        Self {
            grid,
            agents,
            step: 0,
        }
    }

    /// Advance every agent by one tick.
    pub fn update(&mut self) {
        for agent in &mut self.agents {
            agent.update(&self.grid);
        }
        self.step += 1;
    }

    /// Check if a specific agent has reached the goal.
    pub fn agent_at_goal(&self, index: usize) -> bool {
        if let Some(agent) = self.agents.get(index) {
            agent.position() == self.grid.goal
        } else {
            false
        }
    }

    /// How many agents have reached the goal?
    pub fn done_count(&self) -> usize {
        self.agents
            .iter()
            .filter(|a| a.position() == self.grid.goal)
            .count()
    }

    /// Are all agents at the goal?
    pub fn all_done(&self) -> bool {
        self.done_count() == self.agents.len()
    }
}
