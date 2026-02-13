use cognitive_grid::experiments::runner::{run_batch_and_save, ExperimentConfig, AgentType};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    println!("Cognitive Grid — Experiment Sweeps");
    println!("{}", "=".repeat(50));

    let agent_types = [
        (AgentType::AStar, "AStar"),
        (AgentType::Fsm, "FSM"),
        (AgentType::BehaviorTree, "BT"),
    ];

    // ── Sweep 1: Noise levels ──────────────────────────────
    println!("\n[Sweep] Noise levels: 0.0, 0.1, 0.3, 0.5");
    for noise in [0.0, 0.1, 0.3, 0.5] {
        for &(agent_type, label) in &agent_types {
            let config = ExperimentConfig {
                agent_type,
                episodes: 100,
                obstacle_density: 0.2,
                noise,
                ..Default::default()
            };
            let path = run_batch_and_save(&config)?;
            println!("  {} noise={:.1} → {:?}", label, noise, path);
        }
    }

    // ── Sweep 2: A* planning limits ────────────────────────
    println!("\n[Sweep] A* planning limits: None, 50, 20, 5");
    for limit in [None, Some(50), Some(20), Some(5)] {
        let config = ExperimentConfig {
            agent_type: AgentType::AStar,
            episodes: 100,
            obstacle_density: 0.2,
            planning_limit: limit,
            ..Default::default()
        };
        let path = run_batch_and_save(&config)?;
        println!("  AStar planning_limit={:?} → {:?}", limit, path);
    }

    // ── Sweep 3: Memory capacity ───────────────────────────
    println!("\n[Sweep] Memory capacity: 0, 5, 20, 100");
    for mem in [0, 5, 20, 100] {
        for &(agent_type, label) in &agent_types {
            let config = ExperimentConfig {
                agent_type,
                episodes: 100,
                obstacle_density: 0.2,
                memory_capacity: mem,
                ..Default::default()
            };
            let path = run_batch_and_save(&config)?;
            println!("  {} memory={} → {:?}", label, mem, path);
        }
    }

    // ── Sweep 4: Decay rates ───────────────────────────────
    println!("\n[Sweep] Decay rates with noise=0.5: 1.0, 0.99, 0.95");
    for decay in [1.0, 0.99, 0.95] {
        for &(agent_type, label) in &agent_types {
            let config = ExperimentConfig {
                agent_type,
                episodes: 100,
                obstacle_density: 0.2,
                noise: 0.5,
                decay_rate: decay,
                ..Default::default()
            };
            let path = run_batch_and_save(&config)?;
            println!("  {} decay={:.2} → {:?}", label, decay, path);
        }
    }

    println!("\n{}", "=".repeat(50));
    println!("All sweeps completed.");
    Ok(())
}
