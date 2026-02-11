use cognitive_grid::experiments::runner::{
    run_batch_and_save, AgentType, ExperimentConfig,
};

fn main() {
    // Base configuration for experiments.
    let mut cfg = ExperimentConfig::default();
    cfg.episodes = 50;
    cfg.max_steps = 500;
    cfg.obstacle_density = 0.2;

    let agents = [
        AgentType::Fsm,
        AgentType::AStar,
        AgentType::BehaviorTree,
    ];

    for agent in agents {
        cfg.agent_type = agent;
        println!("Running experiments for {:?} agent...", agent);

        match run_batch_and_save(&cfg) {
            Ok(path) => println!("  -> Results written to {:?}", path),
            Err(e) => eprintln!("  -> Failed to run experiments: {e}"),
        }
    }
}

