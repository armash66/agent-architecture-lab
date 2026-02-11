use serde::Serialize;

/// Summary of a single episode/run of an agent.
#[derive(Debug, Clone, Serialize)]
pub struct EpisodeLog {
    /// Sequential episode index (0-based or 1-based, up to the caller).
    pub episode: u32,
    /// Human-readable agent type, e.g. "FSM", "AStar", "BehaviorTree".
    pub agent_type: String,
    /// Number of steps taken in the episode.
    pub steps: u32,
    /// Whether the agent reached the goal.
    pub success: bool,
    /// Agent's remaining energy at the end of the episode
    /// (0 for agents that do not track energy).
    pub energy_remaining: u32,
}

/// Optional per-step log for more detailed analysis.
#[derive(Debug, Clone, Serialize)]
pub struct StepLog {
    pub episode: u32,
    pub step: u32,
    pub x: usize,
    pub y: usize,
    pub energy: u32,
}

/// Write a collection of episode summaries to a CSV file.
///
/// This creates/overwrites the file at `path`.
pub fn write_episode_logs_csv<P: AsRef<std::path::Path>>(
    path: P,
    logs: &[EpisodeLog],
) -> Result<(), Box<dyn std::error::Error>> {
    let mut wtr = csv::Writer::from_path(path)?;
    for log in logs {
        wtr.serialize(log)?;
    }
    wtr.flush()?;
    Ok(())
}

