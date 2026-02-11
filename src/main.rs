use rand::Rng;
use std::{thread, time::Duration};

/// A simple 2D grid with a single goal cell.
struct Grid {
    width: usize,
    height: usize,
    goal: Position,
}

/// A position on the grid.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Position {
    x: usize,
    y: usize,
}

/// The agent that moves around the grid.
struct Agent {
    pos: Position,
}

/// The world contains the grid and the agent.
struct World {
    grid: Grid,
    agent: Agent,
}

impl Grid {
    fn new(width: usize, height: usize, goal: Position) -> Self {
        Self {
            width,
            height,
            goal,
        }
    }
}

impl World {
    /// Create a new world with the agent starting at (0, 0)
    /// and the goal at the bottom-right corner.
    fn new(width: usize, height: usize) -> Self {
        let agent_pos = Position { x: 0, y: 0 };
        let goal = Position {
            x: width.saturating_sub(1),
            y: height.saturating_sub(1),
        };

        let grid = Grid::new(width, height, goal);
        let agent = Agent { pos: agent_pos };

        Self { grid, agent }
    }

    /// Has the agent reached the goal cell?
    fn has_reached_goal(&self) -> bool {
        self.agent.pos == self.grid.goal
    }

    /// Move the agent one step in a random valid direction.
    fn step(&mut self) {
        let mut rng = rand::thread_rng();

        loop {
            let dir = rng.gen_range(0..4);
            let mut new_pos = self.agent.pos;

            match dir {
                // left
                0 if new_pos.x > 0 => new_pos.x -= 1,
                // right
                1 if new_pos.x + 1 < self.grid.width => new_pos.x += 1,
                // up
                2 if new_pos.y > 0 => new_pos.y -= 1,
                // down
                3 if new_pos.y + 1 < self.grid.height => new_pos.y += 1,
                _ => {
                    // invalid move (would go out of bounds), pick another direction
                    continue;
                }
            }

            self.agent.pos = new_pos;
            break;
        }
    }

    /// Print a simple ASCII representation of the grid,
    /// showing the agent and the goal.
    fn print(&self, step: usize) {
        println!("Step {}", step);

        for y in 0..self.grid.height {
            for x in 0..self.grid.width {
                let pos = Position { x, y };

                if pos == self.agent.pos {
                    print!("A ");
                } else if pos == self.grid.goal {
                    print!("G ");
                } else {
                    print!(". ");
                }
            }
            println!();
        }

        println!();
    }
}

fn main() {
    // Minimal Stage 1 world:
    // - Grid
    // - Agent position
    // - Goal cell
    // - Simple game loop with random movement
    let mut world = World::new(10, 5);
    let mut step = 0usize;

    loop {
        world.print(step);

        if world.has_reached_goal() {
            println!(
                "Agent reached the goal at ({}, {}) in {} steps!",
                world.grid.goal.x, world.grid.goal.y, step
            );
            break;
        }

        world.step();
        step += 1;

        // Slow things down so you can see the movement.
        thread::sleep(Duration::from_millis(200));
    }
}
