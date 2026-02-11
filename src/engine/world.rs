use crate::agents::fsm::{FSMAgent, FSMState};

/// A position on the grid.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

/// A simple 2D grid with a single goal cell.
pub struct Grid {
    pub width: usize,
    pub height: usize,
    pub goal: Position,
}

/// The world contains the grid and the FSM agent.
pub struct World {
    pub grid: Grid,
    pub agent: FSMAgent,
    pub step: usize,
}

impl Grid {
    pub fn new(width: usize, height: usize, goal: Position) -> Self {
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
    pub fn new(width: usize, height: usize) -> Self {
        let start_pos = Position { x: 0, y: 0 };
        let goal = Position {
            x: width.saturating_sub(1),
            y: height.saturating_sub(1),
        };

        let grid = Grid::new(width, height, goal);
        let agent = FSMAgent::new(start_pos.x, start_pos.y);

        Self {
            grid,
            agent,
            step: 0,
        }
    }

    /// Has the agent reached the goal cell according to its FSM state?
    pub fn has_reached_goal(&self) -> bool {
        self.agent.state() == FSMState::FoundGoal
    }

    /// Advance the world by one tick: update the FSM agent.
    pub fn update(&mut self) {
        self.agent.update(&self.grid);
        self.step += 1;
    }

    /// Print a simple ASCII representation of the grid,
    /// showing the agent and the goal.
    pub fn print(&self) {
        println!(
            "Step {} | state: {:?} | energy: {}",
            self.step,
            self.agent.state(),
            self.agent.energy()
        );

        for y in 0..self.grid.height {
            for x in 0..self.grid.width {
                let pos = Position { x, y };

                if pos == self.agent.position() {
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

