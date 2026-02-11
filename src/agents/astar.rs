use crate::algorithms::astar::find_path;
use crate::engine::world::{Grid, Position};

/// Agent that uses A* pathfinding to move toward the goal.
pub struct AStarAgent {
    pos: Position,
    path: Vec<(usize, usize)>,
    path_index: usize,
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
        }
    }

    pub fn position(&self) -> Position {
        self.pos
    }

    /// Update the agent: if we don't have a path, compute one.
    /// Then advance one step along the path toward the goal.
    pub fn update(&mut self, grid: &Grid) {
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

