use std::{thread, time::Duration};

use cognitive_grid::agents::astar::AStarAgent;
use cognitive_grid::engine::world::World;

fn main() {
    let agent = Box::new(AStarAgent::new(0, 0));
    let mut world = World::new(10, 5, agent);
    let max_steps: usize = 500;

    loop {
        world.print();
        println!("State: {}", world.agent.debug_state());

        if world.has_reached_goal() {
            println!(
                "A* agent reached the goal at ({}, {}) in {} steps!",
                world.grid.goal.x, world.grid.goal.y, world.step
            );
            break;
        } else if world.is_agent_stuck() {
            println!(
                "A* agent determined there is no path to the goal after {} steps.",
                world.step
            );
            break;
        } else if world.step as usize >= max_steps {
            println!(
                "A* demo reached max_steps={} without reaching goal; exiting.",
                max_steps
            );
            break;
        }

        world.update();
        thread::sleep(Duration::from_millis(200));
    }
}
