# Cognitive Grid 🧠

**A modular grid-world framework for comparing FSM, A*, and Behavior Tree agents.**  
**Designed for experimenting with structured decision-making in a controlled environment.**

---

## 🚀 Overview

**Cognitive Grid** is a minimal, high-performance 2D simulation engine written in Rust. It serves as an experimentation lab for cognitive modeling, allowing researchers and developers to implement, compare, and analyze different agent architectures in identical environmental conditions.

The framework provides the scaffolding to answer questions like:
- *How does a Behavior Tree compare to a Finite State Machine in dynamic environments?*
- *What is the cost of A* pathfinding vs. reactive heuristics?*

## ✨ Key Features

- **⚡ Lightweight Engine**: Custom 2D grid world with obstacles, goals, and hazards.
- **🤖 Modular Agents**:
  - **Finite State Machines (FSM)**: Deterministic state-based logic.
  - **A* Pathfinding**: Optimal path planning with Manhattan heuristics.
  - **Behavior Trees (BT)**: Hierarchical, modular decision making (*In Progress*).
- **📊 Metric Logging**: Track energy usage, steps taken, and success rates (*In Progress*).
- **🧪 Experiment Runner**: Headless batch execution for statistical analysis.

## 🛠️ Getting Started

### Prerequisites
- **Rust**: Latest stable version (Install via [rustup.rs](https://rustup.rs/))

### Installation
```bash
git clone https://github.com/armash66/cognitive-grid-lab.git
cd cognitive-grid-lab
cargo build --release
```

### Running the Simulation
To run the default simulation (currently showcasing the FSM agent):
```bash
cargo run --release
```

## 🏗️ Architecture

The project follows a clean separation of concerns:

| Module | Description |
|--------|-------------|
| `src/engine` | Core simulation loop, grid physics, and world state. |
| `src/agents` | Implementations of FSM, A*, and Behavior Tree agents. |
| `src/algorithms` | Generic algorithms like A* search, BFS, etc. |
| `src/logging` | Metrics collection and structured data export. |
| `src/experiments` | Batch runner for conducting multiple trials. |

## 🧪 Experiments

Cognitive Grid includes a batch runner to compare agents:

```bash
cargo run --bin run_experiments
```

This will run 50 episodes for each agent type (FSM, A*, Behavior Tree) and save the results to `experiments/data/`.

### Analyzing Results
The output data is in CSV format, containing:
- `agent_type`: The architecture used.
- `steps`: Steps taken to reach the goal.
- `energy_remaining`: Remaining energy (for FSM/BT).
- `success`: Whether the goal was reached.

## 🗺️ Roadmap

- [x] **Stage 1**: Core Grid Engine (State, World, Game Loop)
- [x] **Stage 2**: Finite State Machine (FSM) Agent
- [x] **Stage 3**: A* Pathfinding Agent
- [x] **Stage 4**: Behavior Tree Agent
- [x] **Stage 5**: Structured Logging & Experiment Runner

## 📄 License

This project is open source and available under the [MIT License](LICENSE).
