use std::error::Error;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use rand::Rng;

use crate::agents::astar::AStarAgent;
use crate::agents::behavior_tree::BehaviorTreeAgent;
use crate::agents::fsm::{FSMAgent, FSMState};
use crate::engine::world::{Grid, Position};
use crate::logging::metrics::{write_episode_logs_csv, EpisodeLog};

/// Which agent implementation to evaluate.
#[derive(Debug, Clone, Copy)]
pub enum AgentType {
    Fsm,
    AStar,
    BehaviorTree,
}

/// Configuration for a batch of episodes.
pub struct ExperimentConfig {
    pub episodes: u32,
    pub grid_width: usize,
    pub grid_height: usize,
    /// Probability (0.0–1.0) that a non-start/non-goal cell is an obstacle.
    pub obstacle_density: f32,
    pub agent_type: AgentType,
    /// Maximum steps per episode before we declare failure.
    pub max_steps: u32,
}

impl Default for ExperimentConfig {
    fn default() -> Self {
        Self {
            episodes: 100,
            grid_width: 10,
            grid_height: 5,
            obstacle_density: 0.0,
            agent_type: AgentType::AStar,
            max_steps: 500,
        }
    }
}

/// Run a batch of episodes and save a CSV summary under
/// `experiments/data/<timestamp>_results.csv`.
///
/// Returns the path of the CSV file that was written.
pub fn run_batch_and_save(config: &ExperimentConfig) -> Result<PathBuf, Box<dyn Error>> {
    let logs = run_batch(config);

    let mut dir = PathBuf::from("experiments");
    dir.push("data");
    fs::create_dir_all(&dir)?;

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_secs();
    let filename = format!("{}_results.csv", timestamp);
    let mut path = dir;
    path.push(filename);

    write_episode_logs_csv(&path, &logs)?;
    Ok(path)
}

/// Run a batch of episodes and return the collected episode logs.
pub fn run_batch(config: &ExperimentConfig) -> Vec<EpisodeLog> {
    let mut logs = Vec::with_capacity(config.episodes as usize);

    for episode in 0..config.episodes {
        let log = run_single_episode(config, episode);
        logs.push(log);
    }

    logs
}

fn run_single_episode(config: &ExperimentConfig, episode_idx: u32) -> EpisodeLog {
    let mut steps = 0u32;
    let mut success = false;
    let energy_remaining: u32;

    match config.agent_type {
        AgentType::Fsm => {
            let goal = Position {
                x: config.grid_width - 1,
                y: config.grid_height - 1,
            };
            let grid = make_grid_with_obstacles(config, goal);
            let mut agent = FSMAgent::new(0, 0);

            while steps < config.max_steps {
                if agent.state() == FSMState::FoundGoal {
                    success = true;
                    break;
                }
                agent.update(&grid);
                steps += 1;
            }

            energy_remaining = agent.energy();
        }
        AgentType::AStar => {
            let goal = Position {
                x: config.grid_width - 1,
                y: config.grid_height - 1,
            };
            let grid = make_grid_with_obstacles(config, goal);
            let mut agent = AStarAgent::new(0, 0);

            while steps < config.max_steps {
                if agent.position() == grid.goal {
                    success = true;
                    break;
                }
                agent.update(&grid);
                steps += 1;
            }

            // A* agent currently does not track energy.
            energy_remaining = 0;
        }
        AgentType::BehaviorTree => {
            let goal = Position {
                x: config.grid_width - 1,
                y: config.grid_height - 1,
            };
            let grid = make_grid_with_obstacles(config, goal);
            let mut agent = BehaviorTreeAgent::new(0, 0);

            while steps < config.max_steps {
                if agent.position() == grid.goal {
                    success = true;
                    break;
                }
                agent.update(&grid);
                steps += 1;
            }

            energy_remaining = agent.energy();
        }
    }

    EpisodeLog {
        episode: episode_idx,
        agent_type: match config.agent_type {
            AgentType::Fsm => "FSM".to_string(),
            AgentType::AStar => "AStar".to_string(),
            AgentType::BehaviorTree => "BehaviorTree".to_string(),
        },
        steps,
        success,
        energy_remaining,
    }
}

fn make_grid_with_obstacles(config: &ExperimentConfig, goal: Position) -> Grid {
    let mut rng = rand::thread_rng();
    let mut obstacles = Vec::new();

    for y in 0..config.grid_height {
        for x in 0..config.grid_width {
            // Always keep start and goal walkable.
            if (x == 0 && y == 0) || (x == goal.x && y == goal.y) {
                continue;
            }

            if rng.gen_range(0.0f32..1.0f32) < config.obstacle_density {
                obstacles.push((x, y));
            }
        }
    }

    Grid::with_obstacles(config.grid_width, config.grid_height, goal, &obstacles)
}

