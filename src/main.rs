use std::{thread, time::Duration};

mod agents;
mod engine;

use engine::world::World;

fn main() {
    // Stage 2:
    // - World + Grid moved into engine::world
    // - Agent is now an FSM-based FSMAgent
    // - Game loop calls World::update(), which updates the FSM
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
