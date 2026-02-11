## Cognitive Grid

An experimental playground for comparing different agent control architectures on a simple grid world:

- **Stage 1**: Random walk agent
- **Stage 2**: Finite State Machine (FSM)
- **Stage 3**: A* pathfinding agent
- **Stage 4**: Behavior Tree (BT) agent

### Status / Badges

> Build status badge can be added here once CI is configured, for example a GitHub Actions or other CI badge.

### Quick Start

- **Build**:

```bash
cargo build
```

- **Run all demos (debug)**:

```bash
cargo run
```

This prints a list of available demo binaries. You can run each stage explicitly:

- **Stage 1 – Random walk**:

```bash
cargo run --bin stage1_random
```

- **Stage 2 – FSM agent**:

```bash
cargo run --bin stage2_fsm
```

- **Stage 3 – A* pathfinding**:

```bash
cargo run --bin stage3_astar
```

- **Stage 4 – Behavior Tree**:

```bash
cargo run --bin stage4_behavior_tree
```

For faster experiments, use release mode:

```bash
cargo run --release --bin stage3_astar
```

### Architecture Overview

#### Modules

- **`agents`**: Different agent implementations:
  - `fsm`: classic FSM-based agent.
  - `astar`: agent driven by A* pathfinding.
  - `behavior_tree`: agent driven by a simple Behavior Tree.
- **`engine`**:
  - `world`: grid representation (`Grid`, `Position`) and the A* world used by the Stage 3 demo.
- **`algorithms`**:
  - `astar`: grid-based A* pathfinder.
- **`logging`**:
  - `metrics`: episode-level and (optional) step-level logging utilities, with CSV export.
- **`experiments`**:
  - `runner`: batch runner for automated experiments over multiple episodes.
- **`src/bin`**:
  - `stage1_random.rs`, `stage2_fsm.rs`, `stage3_astar.rs`, `stage4_behavior_tree.rs`: one entrypoint per stage.

#### High-level data flow

```mermaid
graph TD
    A[Binary: stage*_*.rs] --> B[World / Grid]
    B --> C[Agent (FSM / A* / BT)]
    C --> B
    B --> D[Logging::metrics]
    D --> E[experiments/data/*.csv]
```

### Experiments and Logging

The `experiments::runner` module allows running many episodes and saving results for later analysis.

- **Configuration** is done via `ExperimentConfig`:
  - `episodes`: number of episodes.
  - `grid_width`, `grid_height`.
  - `obstacle_density`: probability that a non-start/non-goal cell becomes an obstacle.
  - `agent_type`: `FSM`, `AStar`, or `BehaviorTree`.
  - `max_steps`: per-episode step cap.

- **Episode metrics** are stored in `EpisodeLog`:
  - `episode`, `agent_type`, `steps`, `success`, `energy_remaining`.

#### Example: run a batch and save CSV

From a custom binary or a REPL, you can do something like:

```rust
use cognitive_grid::experiments::runner::{run_batch_and_save, ExperimentConfig, AgentType};

fn main() {
    let mut cfg = ExperimentConfig::default();
    cfg.episodes = 100;
    cfg.agent_type = AgentType::AStar;
    cfg.obstacle_density = 0.2;

    let path = run_batch_and_save(&cfg).expect("experiment failed");
    println!("Results written to {:?}", path);
}
```

This will create a file like `experiments/data/<timestamp>_results.csv` containing one row per episode.

### Results (Placeholder)

This section is intended for:

- **Comparison tables** of success rates and average steps for FSM vs A* vs BT.
- **Plots** (generated externally) of:
  - success rate vs. obstacle density,
  - steps-to-goal distributions,
  - energy usage over time for BT/FSM.

You can generate these plots by loading the CSVs from `experiments/data` into a notebook or plotting tool of your choice.

