use rand::Rng;
use crate::algorithms::astar::find_path;
use crate::engine::world::{Grid, Position};
use super::memory::SpatialMemory;

/// Agent that uses A* pathfinding to move toward the goal.
pub struct AStarAgent {
    pos: Position,
    path: Vec<(usize, usize)>,
    path_index: usize,
    /// Set to true if we determined there is no path to the goal
    /// under the current grid configuration.
    stuck: bool,
    /// Max node expansions for bounded A*. `None` = unlimited.
    planning_limit: Option<usize>,
    noise: f32,
    exploration_rate: f32,
    decay_rate: f32,
    memory: SpatialMemory,
    noise_triggered: bool,
}

impl AStarAgent {
    pub fn new(start_x: usize, start_y: usize) -> Self {
        Self {
            pos: Position {
                x: start_x,
                y: start_y,
            },
            path: Vec::new(),
            path_index: 0,
            stuck: false,
            planning_limit: None,
            noise: 0.0,
            exploration_rate: 1.0,
            decay_rate: 1.0,
            memory: SpatialMemory::new(0),
            noise_triggered: false,
        }
    }

    /// Create an A* agent with a bounded planning limit.
    pub fn with_planning_limit(start_x: usize, start_y: usize, limit: usize) -> Self {
        Self {
            planning_limit: Some(limit),
            ..Self::new(start_x, start_y)
        }
    }

    /// Create an A* agent with full cognitive parameters.
    pub fn with_config(
        start_x: usize,
        start_y: usize,
        planning_limit: Option<usize>,
        noise: f32,
        memory_capacity: usize,
        decay_rate: f32,
    ) -> Self {
        Self {
            planning_limit,
            noise,
            decay_rate,
            memory: SpatialMemory::new(memory_capacity),
            noise_triggered: false,
            ..Self::new(start_x, start_y)
        }
    }

    pub fn position(&self) -> Position {
        self.pos
    }

    /// Whether the agent has determined that no path exists and stopped trying.
    pub fn is_stuck(&self) -> bool {
        self.stuck
    }

    /// Update the agent: if we don't have a path, compute one.
    /// Then advance one step along the path toward the goal.
    pub fn update(&mut self, grid: &Grid) {
        self.noise_triggered = false;
        // Record current position in memory.
        self.memory.record(self.pos);

        // Decay exploration rate.
        self.exploration_rate *= self.decay_rate;
        // ... (existing update logic) ...
        // If we already know there's no path, do nothing.
        if self.stuck {
            return;
        }

        // Already at goal.
        if self.pos == grid.goal {
            return;
        }

        // Plan a path if needed or if we've exhausted the previous plan.
        if self.path.is_empty() || self.path_index + 1 >= self.path.len() {
            let start = (self.pos.x, self.pos.y);
            let goal = (grid.goal.x, grid.goal.y);

            match find_path(start, goal, grid, self.planning_limit) {
                Some(path) => {
                    let path: Vec<(usize, usize)> = path;
                    let path_len: usize = path.len();
                    println!(
                        "A*: Planned path from {:?} to {:?} with length {}",
                        start,
                        goal,
                        path_len
                    );
                    self.path = path;
                    self.path_index = 0;
                }
                None => {
                    println!("A*: No path found from {:?} to {:?}", start, goal);
                    // Mark as stuck so we don't keep re-planning every tick.
                    self.stuck = true;
                    return;
                }
            }
        }

        // Move along the path by one step, if possible.
        if self.path_index + 1 < self.path.len() {
            // Decision noise (modulated by exploration rate).
            let effective_noise = self.noise * self.exploration_rate;
            let mut rng = rand::thread_rng();
            if effective_noise > 0.0 && rng.r#gen::<f32>() < effective_noise {
                if let Some((nx, ny)) = grid.random_walkable_neighbor(self.pos.x, self.pos.y) {
                    self.pos = Position { x: nx, y: ny };
                    // Invalidate path so we re-plan next tick.
                    self.path.clear();
                    self.noise_triggered = true;
                    println!("A*: Noise! Random move to ({}, {})", nx, ny);
                    return;
                }
            }

            self.path_index += 1;
            let (nx, ny) = self.path[self.path_index];
            self.pos = Position { x: nx, y: ny };
            println!("A*: Moving to ({}, {})", nx, ny);
        }
    }
}

impl super::Agent for AStarAgent {
    fn update(&mut self, grid: &Grid) {
        self.update(grid);
    }

    fn position(&self) -> Position {
        self.pos
    }

    fn name(&self) -> &'static str {
        "AStar"
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn is_stuck(&self) -> bool {
        self.stuck
    }

    fn debug_state(&self) -> String {
        if self.stuck {
            "Stuck".to_string()
        } else {
            format!("Path len: {}", self.path.len())
        }
        }


    fn did_noise_trigger(&self) -> bool {
        self.noise_triggered
    }

    fn planning_radius(&self) -> Option<f32> {
        self.planning_limit.map(|l| l as f32)
    }
}

