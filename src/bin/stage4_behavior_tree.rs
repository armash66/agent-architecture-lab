use std::{thread, time::Duration};

use cognitive_grid::agents::behavior_tree::BehaviorTreeAgent;
use cognitive_grid::engine::world::World;

fn main() {
    // Create BT agent starting at (0,0)
    let agent = Box::new(BehaviorTreeAgent::new(0, 0));
    let mut world = World::new(10, 8, agent);

    loop {
        world.print();
        println!(
            "BT Info: Energy={:?}", 
            world.agent.energy()
        );

        if world.has_reached_goal() {
            println!(
                "BT agent reached goal at ({}, {}) in {} steps.",
                world.grid.goal.x,
                world.grid.goal.y,
                world.step
            );
            break;
        }

        world.update();
        thread::sleep(Duration::from_millis(200));
    }
}
