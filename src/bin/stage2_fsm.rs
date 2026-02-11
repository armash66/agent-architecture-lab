use std::{thread, time::Duration};

use cognitive_grid::agents::fsm::{FSMAgent, FSMState};
use cognitive_grid::engine::world::{Grid, Position};

struct WorldFsm {
    grid: Grid,
    agent: FSMAgent,
    step: usize,
}

impl WorldFsm {
    fn new(width: usize, height: usize) -> Self {
        let start = Position { x: 0, y: 0 };
        let goal = Position { x: width - 1, y: height - 1 };
        let grid = Grid::new(width, height, goal);
        let agent = FSMAgent::new(start.x, start.y);
        Self { grid, agent, step: 0 }
    }

    fn has_reached_goal(&self) -> bool {
        self.agent.state() == FSMState::FoundGoal
    }

    fn update(&mut self) {
        self.agent.update(&self.grid);
        self.step += 1;
    }

    fn print(&self) {
        println!(
            "Stage 2 – FSM | step {} | state: {:?} | energy: {}",
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

fn main() {
    let mut world = WorldFsm::new(10, 5);

    loop {
        world.print();

        if world.has_reached_goal() {
            println!(
                "FSM agent reached goal at ({}, {}) in {} steps",
                world.grid.goal.x, world.grid.goal.y, world.step
            );
            break;
        }

        world.update();
        thread::sleep(Duration::from_millis(200));
    }
}

