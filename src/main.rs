use std::{thread, time::Duration};

mod agents;
mod engine;
mod algorithms;

use engine::world::World;

fn main() {
    let mut world = World::new(10, 5);

    loop {
        world.print();

        if world.has_reached_goal() {
            println!(
                "Agent reached the goal at ({}, {}) in {} steps!",
                world.grid.goal.x, world.grid.goal.y, world.step
            );
            break;
        }

        world.update();

        // Slow things down so you can see the movement.
        thread::sleep(Duration::from_millis(200));
    }
}
