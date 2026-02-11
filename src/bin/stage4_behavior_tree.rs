use std::{thread, time::Duration};

use cognitive_grid::agents::behavior_tree::BehaviorTreeAgent;
use cognitive_grid::engine::world::{Grid, Position};

struct WorldBt {
    grid: Grid,
    agent: BehaviorTreeAgent,
    step: usize,
}

impl WorldBt {
    fn new(width: usize, height: usize) -> Self {
        let start = Position { x: 0, y: 0 };
        let goal = Position {
            x: width - 1,
            y: height - 1,
        };
        let grid = Grid::new(width, height, goal);
        let agent = BehaviorTreeAgent::new(start.x, start.y);

        Self { grid, agent, step: 0 }
    }

    fn has_reached_goal(&self) -> bool {
        self.agent.position() == self.grid.goal
    }

    fn update(&mut self) {
        self.agent.update(&self.grid);
        self.step += 1;
    }

    fn print(&self) {
        println!(
            "Stage 4 – Behavior Tree | step {} | energy={}",
            self.step,
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
    let mut world = WorldBt::new(10, 5);

    loop {
        world.print();

        if world.has_reached_goal() {
            println!(
                "BT agent reached goal at ({}, {}) in {} steps with energy {}",
                world.grid.goal.x,
                world.grid.goal.y,
                world.step,
                world.agent.energy()
            );
            break;
        }

        world.update();
        thread::sleep(Duration::from_millis(200));
    }
}

