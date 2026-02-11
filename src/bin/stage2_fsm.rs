use std::{thread, time::Duration};

use cognitive_grid::agents::fsm::FSMAgent;
use cognitive_grid::engine::world::World;

fn main() {
    let agent = Box::new(FSMAgent::new(0, 0));
    let mut world = World::new(10, 5, agent);

    loop {
        world.print();

        // Print extra debug info using trait methods
        println!(
            "Extra Info: State=[{}] Energy=[{:?}]",
            world.agent.debug_state(),
            world.agent.energy()
        );

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
