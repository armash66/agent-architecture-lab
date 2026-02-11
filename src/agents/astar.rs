use crate::algorithms::astar::find_path;
use crate::engine::world::{Grid, Position};

/// Agent that uses A* pathfinding to move toward the goal.
pub struct AStarAgent {
    pos: Position,
    path: Vec<(usize, usize)>,
    path_index: usize,
    /// Set to true if we determined there is no path to the goal
    /// under the current grid configuration.
    stuck: bool,
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

            match find_path(start, goal, grid) {
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
}

