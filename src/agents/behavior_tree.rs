use rand::Rng;

use crate::engine::world::{Grid, Position};

/// Status returned by behavior tree nodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Success,
    Failure,
    Running,
}

/// Behavior Tree node.
///
/// This keeps things simple by using function pointers for
/// conditions and actions, and recursive enums for Sequence
/// and Selector.
pub enum Node {
    /// Runs children in order; fails on first Failure,
    /// succeeds only if all children succeed.
    Sequence(Vec<Node>),
    /// Runs children in order; succeeds on first Success,
    /// fails only if all children fail.
    Selector(Vec<Node>),
    /// Condition that checks read-only agent/world state.
    Condition(fn(&BehaviorTreeAgent, &Grid) -> bool),
    /// Action that can modify agent state.
    Action(fn(&mut BehaviorTreeAgent, &Grid) -> Status),
}

impl Node {
    pub fn tick(&mut self, agent: &mut BehaviorTreeAgent, grid: &Grid) -> Status {
        match self {
            Node::Sequence(children) => {
                for child in children.iter_mut() {
                    match child.tick(agent, grid) {
                        Status::Success => continue,
                        Status::Failure => return Status::Failure,
                        Status::Running => return Status::Running,
                    }
                }
                Status::Success
            }
            Node::Selector(children) => {
                for child in children.iter_mut() {
                    match child.tick(agent, grid) {
                        Status::Success => return Status::Success,
                        Status::Running => return Status::Running,
                        Status::Failure => continue,
                    }
                }
                Status::Failure
            }
            Node::Condition(pred) => {
                if pred(agent, grid) {
                    Status::Success
                } else {
                    Status::Failure
                }
            }
            Node::Action(act) => act(agent, grid),
        }
    }
}

/// Simple behavior-tree driven agent.
///
/// For now it has:
/// - Position on the grid
/// - Energy level
/// - A hard-coded tree:
///     Selector(
///       Sequence(IsHungry, MoveTowardsGoal),
///       Wander
///     )
pub struct BehaviorTreeAgent {
    pos: Position,
    energy: u32,
    root: Node,
}

impl BehaviorTreeAgent {
    pub fn new(start_x: usize, start_y: usize) -> Self {
        // Build tree using top-level helper functions below.
        let root = Node::Selector(vec![
            Node::Sequence(vec![
                Node::Condition(is_hungry),
                Node::Action(move_towards_goal),
            ]),
            Node::Action(wander),
        ]);

        Self {
            pos: Position {
                x: start_x,
                y: start_y,
            },
            energy: 100,
            root,
        }
    }

    pub fn position(&self) -> Position {
        self.pos
    }

    pub fn energy(&self) -> u32 {
        self.energy
    }

    /// Advance the behavior tree by one tick.
    pub fn update(&mut self, grid: &Grid) {
        // Work around Rust's borrow checker by temporarily taking ownership
        // of the root node while ticking.
        let mut root = std::mem::replace(&mut self.root, Node::Action(noop_action));
        let status = root.tick(self, grid);
        self.root = root;

        println!(
            "BT tick -> {:?} | pos=({}, {}) | energy={}",
            status, self.pos.x, self.pos.y, self.energy
        );
    }
}

impl super::Agent for BehaviorTreeAgent {
    fn update(&mut self, grid: &Grid) {
        self.update(grid);
    }

    fn position(&self) -> Position {
        self.pos
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn energy(&self) -> Option<u32> {
        Some(self.energy)
    }

    fn debug_state(&self) -> String {
        "Running".to_string() // BT doesn't have a single state enum like FSM
    }
}

// === Conditions and Actions used in the default tree ===

fn is_hungry(agent: &BehaviorTreeAgent, _grid: &Grid) -> bool {
    agent.energy < 50
}

fn manhattan(a: Position, b: Position) -> u32 {
    let dx = a.x.abs_diff(b.x);
    let dy = a.y.abs_diff(b.y);
    (dx + dy) as u32
}

/// Move greedily toward the goal, consuming a small amount of energy.
fn move_towards_goal(agent: &mut BehaviorTreeAgent, grid: &Grid) -> Status {
    if agent.pos == grid.goal {
        // "Eat": recover energy when at the goal cell.
        let before = agent.energy;
        agent.energy = (agent.energy + 20).min(100);
        println!(
            "BT: At goal, recovering energy {} -> {}",
            before, agent.energy
        );
        return Status::Success;
    }

    let current = agent.pos;
    let goal = grid.goal;
    let current_h = manhattan(current, goal);

    let candidates = [
        Position {
            x: current.x.wrapping_sub(1),
            y: current.y,
        },
        Position {
            x: current.x + 1,
            y: current.y,
        },
        Position {
            x: current.x,
            y: current.y.wrapping_sub(1),
        },
        Position {
            x: current.x,
            y: current.y + 1,
        },
    ];

    let mut best: Option<(Position, u32)> = None;

    for cand in candidates {
        if !grid.is_walkable(cand.x, cand.y) {
            continue;
        }
        let h = manhattan(cand, goal);
        if h < current_h {
            match best {
                Some((_, best_h)) if h >= best_h => {}
                _ => best = Some((cand, h)),
            }
        }
    }

    if let Some((next, _)) = best {
        agent.pos = next;
        agent.energy = agent.energy.saturating_sub(1);
        println!(
            "BT: Moving toward goal -> ({}, {}), energy={}",
            next.x, next.y, agent.energy
        );
        Status::Success
    } else {
        // No improving move found.
        Status::Failure
    }
}

/// Wander randomly, consuming a bit of energy.
fn wander(agent: &mut BehaviorTreeAgent, grid: &Grid) -> Status {
    let mut rng = rand::thread_rng();

    for _ in 0..8 {
        let dir = rng.gen_range(0..4);
        let mut next = agent.pos;

        match dir {
            0 if next.x > 0 => next.x -= 1,
            1 => next.x = next.x.saturating_add(1),
            2 if next.y > 0 => next.y -= 1,
            3 => next.y = next.y.saturating_add(1),
            _ => continue,
        }

        if grid.is_walkable(next.x, next.y) {
            agent.pos = next;
            agent.energy = agent.energy.saturating_sub(1);
            println!(
                "BT: Wandering to ({}, {}), energy={}",
                next.x, next.y, agent.energy
            );
            return Status::Success;
        }
    }

    Status::Failure
}

fn noop_action(_agent: &mut BehaviorTreeAgent, _grid: &Grid) -> Status {
    Status::Running
}

// === Unit tests for core BT logic ===

#[cfg(test)]
mod tests {
    use super::*;

    use crate::engine::world::Grid;

    fn dummy_grid() -> Grid {
        Grid::new(1, 1, Position { x: 0, y: 0 })
    }

    fn dummy_agent() -> BehaviorTreeAgent {
        BehaviorTreeAgent::new(0, 0)
    }

    fn cond_true(_agent: &BehaviorTreeAgent, _grid: &Grid) -> bool {
        true
    }

    fn cond_false(_agent: &BehaviorTreeAgent, _grid: &Grid) -> bool {
        false
    }

    fn action_success(_agent: &mut BehaviorTreeAgent, _grid: &Grid) -> Status {
        Status::Success
    }

    fn action_failure(_agent: &mut BehaviorTreeAgent, _grid: &Grid) -> Status {
        Status::Failure
    }

    #[test]
    fn sequence_fails_on_first_failure() {
        let mut root = Node::Sequence(vec![
            Node::Action(action_failure),
            Node::Action(action_success),
        ]);

        let mut agent = dummy_agent();
        let grid = dummy_grid();

        let status = root.tick(&mut agent, &grid);
        assert_eq!(status, Status::Failure);
    }

    #[test]
    fn selector_succeeds_on_first_success() {
        let mut root = Node::Selector(vec![
            Node::Action(action_failure),
            Node::Action(action_success),
        ]);

        let mut agent = dummy_agent();
        let grid = dummy_grid();

        let status = root.tick(&mut agent, &grid);
        assert_eq!(status, Status::Success);
    }

    #[test]
    fn condition_wraps_bool_result() {
        let mut node_true = Node::Condition(cond_true);
        let mut node_false = Node::Condition(cond_false);

        let mut agent = dummy_agent();
        let grid = dummy_grid();

        assert_eq!(node_true.tick(&mut agent, &grid), Status::Success);
        assert_eq!(node_false.tick(&mut agent, &grid), Status::Failure);
    }
}

