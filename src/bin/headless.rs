use cognitive_grid::agents::astar::AStarAgent;
use cognitive_grid::agents::behavior_tree::BehaviorTreeAgent;
use cognitive_grid::agents::fsm::FSMAgent;
use cognitive_grid::agents::Agent;
use cognitive_grid::engine::world::World;

fn main() {
    let agents: Vec<Box<dyn Agent>> = vec![
        Box::new(FSMAgent::new(0, 0)),
        Box::new(AStarAgent::new(0, 0)),
        Box::new(BehaviorTreeAgent::new(0, 0)),
    ];

    let grid_w = 10;
    let grid_h = 5;
    let max_steps: usize = 500;

    println!("Cognitive Grid — Headless Runner");
    println!("Grid: {}x{} | Max steps: {}", grid_w, grid_h, max_steps);
    println!("{}", "-".repeat(50));

    for agent in agents {
        let name = agent.name();
        let mut world = World::new(grid_w, grid_h, agent);
        let mut steps = 0usize;

        while steps < max_steps {
            if world.has_reached_goal() {
                break;
            }
            if world.is_agent_stuck() {
                break;
            }
            world.update();
            steps += 1;
        }

        let success = world.has_reached_goal();
        let energy = world.agent.energy().unwrap_or(0);

        println!(
            "[{}] steps={}, success={}, energy={}",
            name, steps, success, energy
        );
    }

    println!("{}", "-".repeat(50));
    println!("Done.");
}
