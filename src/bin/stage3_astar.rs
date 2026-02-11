use std::{thread, time::Duration};

use cognitive_grid::engine::world::World;

fn main() {
    let mut world = World::new(10, 5);

    loop {
        world.print();

        if world.has_reached_goal() {
            println!(
                "A* agent reached the goal at ({}, {}) in {} steps!",
                world.grid.goal.x, world.grid.goal.y, world.step
            );
            break;
        }

        world.update();
        thread::sleep(Duration::from_millis(200));
    }
}

