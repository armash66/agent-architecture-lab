use rand::Rng;
use std::{thread, time::Duration};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Position {
    x: usize,
    y: usize,
}

struct Grid {
    width: usize,
    height: usize,
    goal: Position,
}

struct World {
    grid: Grid,
    agent: Position,
    step: usize,
}

impl Grid {
    fn new(width: usize, height: usize, goal: Position) -> Self {
        Self { width, height, goal }
    }
}

impl World {
    fn new(width: usize, height: usize) -> Self {
        let start = Position { x: 0, y: 0 };
        let goal = Position { x: width - 1, y: height - 1 };
        let grid = Grid::new(width, height, goal);
        Self {
            grid,
            agent: start,
            step: 0,
        }
    }

    fn has_reached_goal(&self) -> bool {
        self.agent == self.grid.goal
    }

    fn update(&mut self) {
        let mut rng = rand::thread_rng();
        loop {
            let dir = rng.gen_range(0..4);
            let mut next = self.agent;

            match dir {
                0 if next.x > 0 => next.x -= 1,
                1 if next.x + 1 < self.grid.width => next.x += 1,
                2 if next.y > 0 => next.y -= 1,
                3 if next.y + 1 < self.grid.height => next.y += 1,
                _ => continue,
            }

            self.agent = next;
            self.step += 1;
            break;
        }
    }

    fn print(&self) {
        println!("Stage 1 – Random | step {}", self.step);
        for y in 0..self.grid.height {
            for x in 0..self.grid.width {
                let pos = Position { x, y };
                if pos == self.agent {
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
    let mut world = World::new(10, 5);

    loop {
        world.print();

        if world.has_reached_goal() {
            println!(
                "Random agent reached goal at ({}, {}) in {} steps",
                world.grid.goal.x, world.grid.goal.y, world.step
            );
            break;
        }

        world.update();
        thread::sleep(Duration::from_millis(200));
    }
}

