use crate::agents::Agent;

pub use super::position::Position;

/// A simple 2D grid with a single goal cell.
pub struct Grid {
    pub width: usize,
    pub height: usize,
    pub goal: Position,
    tiles: Vec<Vec<bool>>,
}

/// The world contains the grid and a polymorphic agent.
pub struct World {
    pub grid: Grid,
    pub agent: Box<dyn Agent>,
    pub step: usize,
}

impl Grid {
    pub fn new(width: usize, height: usize, goal: Position) -> Self {
        let tiles = vec![vec![true; width]; height];

        Self {
            width,
            height,
            goal,
            tiles,
        }
    }

    /// Convenience constructor for tests or experiments that need obstacles.
    /// `obstacles` is a list of (x, y) cells that are *not* walkable.
    pub fn with_obstacles(
        width: usize,
        height: usize,
        goal: Position,
        obstacles: &[(usize, usize)],
    ) -> Self {
        let mut grid = Self::new(width, height, goal);

        for &(x, y) in obstacles {
            if x < width && y < height && (x != goal.x || y != goal.y) {
                grid.tiles[y][x] = false;
            }
        }

        grid
    }

    /// Return whether the given cell is walkable (in-bounds and not blocked).
    pub fn is_walkable(&self, x: usize, y: usize) -> bool {
        if x >= self.width || y >= self.height {
            return false;
        }
        self.tiles[y][x]
    }
}

impl World {
    /// Create a new world with the given agent.
    pub fn new(width: usize, height: usize, agent: Box<dyn Agent>) -> Self {
        let goal = Position {
            x: width.saturating_sub(1),
            y: height.saturating_sub(1),
        };

        let grid = Grid::new(width, height, goal);

        Self {
            grid,
            agent,
            step: 0,
        }
    }

    /// Has the agent reached the goal cell?
    pub fn has_reached_goal(&self) -> bool {
        let pos = self.agent.position();
        pos == self.grid.goal
    }

    /// Whether the agent is stuck (mostly for A*).
    pub fn is_agent_stuck(&self) -> bool {
        self.agent.is_stuck()
    }

    /// Advance the world by one tick: update the agent.
    pub fn update(&mut self) {
        self.agent.update(&self.grid);
        self.step += 1;
    }

    /// Print a simple ASCII representation of the grid,
    /// showing the agent and the goal.
    pub fn print(&self) {
        println!("Step {} | Agent at {:?}", self.step, self.agent.position());

        for y in 0..self.grid.height {
            for x in 0..self.grid.width {
                let pos = Position { x, y };

                if pos == self.agent.position() {
                    print!("A ");
                } else if pos == self.grid.goal {
                    print!("G ");
                } else {
                    if self.grid.is_walkable(x, y) {
                        print!(". ");
                    } else {
                        print!("# ");
                    }
                }
            }
            println!();
        }

        println!();
    }
}


