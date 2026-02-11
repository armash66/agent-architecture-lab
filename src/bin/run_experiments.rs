use cognitive_grid::experiments::runner::{run_batch_and_save, ExperimentConfig, AgentType};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    println!("Starting Research Experiments...");

    // 1. Run FSM Experiment
    println!("Running FSM Batch...");
    let config_fsm = ExperimentConfig {
        agent_type: AgentType::Fsm,
        episodes: 50,
        obstacle_density: 0.2, // 20% obstacles
        ..Default::default()
    };
    let path = run_batch_and_save(&config_fsm)?;
    println!("Saved FSM results to: {:?}", path);

    // 2. Run A* Experiment
    println!("Running A* Batch...");
    let config_astar = ExperimentConfig {
        agent_type: AgentType::AStar,
        episodes: 50,
        obstacle_density: 0.2,
        ..Default::default()
    };
    let path = run_batch_and_save(&config_astar)?;
    println!("Saved A* results to: {:?}", path);

    // 3. Run BT Experiment
    println!("Running Behavior Tree Batch...");
    let config_bt = ExperimentConfig {
        agent_type: AgentType::BehaviorTree,
        episodes: 50,
        obstacle_density: 0.2,
        ..Default::default()
    };
    let path = run_batch_and_save(&config_bt)?;
    println!("Saved BT results to: {:?}", path);

    println!("All experiments completed successfully.");
    Ok(())
}
