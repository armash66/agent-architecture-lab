use cognitive_grid::agents::astar::AStarAgent;
use cognitive_grid::agents::behavior_tree::BehaviorTreeAgent;
use cognitive_grid::agents::fsm::FSMAgent;
use cognitive_grid::agents::Agent;
use cognitive_grid::engine::multi_world::MultiWorld;
use cognitive_grid::engine::world::{Grid, Position};

fn main() {
    let grid_w = 10;
    let grid_h = 5;
    let max_steps: usize = 500;
    let obstacle_density = 0.15;

    // Build a shared grid with random obstacles.
    let goal = Position {
        x: grid_w - 1,
        y: grid_h - 1,
    };
    let mut grid = Grid::new(grid_w, grid_h, goal);
    grid.scatter_obstacles(obstacle_density);

    let obstacles = grid.obstacle_positions();

    // Create agents with cognitive parameters.
    let agents: Vec<Box<dyn Agent>> = vec![
        Box::new(FSMAgent::with_config(0, 0, 0.15, 10, 0.995)),
        Box::new(AStarAgent::with_config(0, 0, Some(30), 0.1, 10, 0.995)),
        Box::new(BehaviorTreeAgent::with_config(0, 0, 0.15, 10, 0.995)),
    ];

    let agent_names: Vec<&str> = agents.iter().map(|a| a.name()).collect();

    let mut world = MultiWorld::new(grid, agents);

    println!("Cognitive Grid — Multi-Agent Headless Runner");
    println!("Grid: {}x{} | Obstacles: {} | Max steps: {}",
        grid_w, grid_h, obstacles.len(), max_steps);
    println!("{}", "═".repeat(55));

    // Run until all agents reach the goal or we exceed max steps.
    let mut finish_step: Vec<Option<usize>> = vec![None; world.agents.len()];

    while world.step < max_steps {
        // Check who just finished.
        for (i, agent) in world.agents.iter().enumerate() {
            if finish_step[i].is_none() && agent.position() == world.grid.goal {
                finish_step[i] = Some(world.step);
                println!("  ✓ {} reached goal at step {}", agent_names[i], world.step);
            }
        }

        if world.all_done() {
            break;
        }

        world.update();
    }

    // Final check (in case an agent reached goal on the last tick).
    for (i, agent) in world.agents.iter().enumerate() {
        if finish_step[i].is_none() && agent.position() == world.grid.goal {
            finish_step[i] = Some(world.step);
        }
    }

    // Print summary.
    println!("{}", "═".repeat(55));
    println!("{:<15} {:>6} {:>8} {:>8}",
        "Agent", "Steps", "Success", "Energy");
    println!("{}", "─".repeat(55));
    for (i, agent) in world.agents.iter().enumerate() {
        let success = finish_step[i].is_some();
        let steps = finish_step[i].unwrap_or(world.step);
        let energy = agent.energy().unwrap_or(0);
        println!("{:<15} {:>6} {:>8} {:>8}",
            agent_names[i], steps, success, energy);
    }
    println!("{}", "═".repeat(55));
}
