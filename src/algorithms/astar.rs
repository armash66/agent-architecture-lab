use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};

use crate::engine::world::Grid;

/// Internal A* node stored in the open set.
#[derive(Clone, Debug, Eq, PartialEq)]
struct Node {
    position: (usize, usize),
    g_cost: u32,
    h_cost: u32,
}

impl Node {
    fn f_cost(&self) -> u32 {
        self.g_cost + self.h_cost
    }
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        // BinaryHeap is max-first; invert comparison so that the
        // node with the *smallest* f_cost comes out first.
        other
            .f_cost()
            .cmp(&self.f_cost())
            .then_with(|| other.h_cost.cmp(&self.h_cost))
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn manhattan(a: (usize, usize), b: (usize, usize)) -> u32 {
    let dx = a.0.abs_diff(b.0);
    let dy = a.1.abs_diff(b.1);
    (dx + dy) as u32
}

/// Standard A* pathfinding on the provided grid.
///
/// Returns a path of (x, y) coordinates from `start` to `goal`,
/// including both endpoints, or `None` if no path exists.
pub fn find_path(
    start: (usize, usize),
    goal: (usize, usize),
    grid: &Grid,
) -> Option<Vec<(usize, usize)>> {
    if !grid.is_walkable(start.0, start.1) || !grid.is_walkable(goal.0, goal.1) {
        return None;
    }

    let mut open_set = BinaryHeap::new();
    let mut came_from: HashMap<(usize, usize), (usize, usize)> = HashMap::new();
    let mut g_score: HashMap<(usize, usize), u32> = HashMap::new();
    let mut closed: HashSet<(usize, usize)> = HashSet::new();

    g_score.insert(start, 0);

    open_set.push(Node {
        position: start,
        g_cost: 0,
        h_cost: manhattan(start, goal),
    });

    while let Some(current) = open_set.pop() {
        let current_pos = current.position;

        if current_pos == goal {
            // Reconstruct path.
            let mut path = Vec::new();
            let mut p = current_pos;
            path.push(p);
            while let Some(prev) = came_from.get(&p) {
                p = *prev;
                path.push(p);
            }
            path.reverse();
            return Some(path);
        }

        if closed.contains(&current_pos) {
            continue;
        }
        closed.insert(current_pos);

        let current_g = *g_score.get(&current_pos).unwrap_or(&u32::MAX);

        // 4-directional neighbors.
        let (cx, cy) = current_pos;
        let neighbors = [
            (cx.wrapping_sub(1), cy),     // left (checked below for bounds)
            (cx + 1, cy),                  // right
            (cx, cy.wrapping_sub(1)),     // up
            (cx, cy + 1),                  // down
        ];

        for &(nx, ny) in &neighbors {
            if nx >= grid.width || ny >= grid.height {
                continue;
            }
            if !grid.is_walkable(nx, ny) {
                continue;
            }

            let neighbor_pos = (nx, ny);
            if closed.contains(&neighbor_pos) {
                continue;
            }

            let tentative_g = current_g.saturating_add(1);
            let best_known_g = *g_score.get(&neighbor_pos).unwrap_or(&u32::MAX);

            if tentative_g < best_known_g {
                g_score.insert(neighbor_pos, tentative_g);
                came_from.insert(neighbor_pos, current_pos);

                let h = manhattan(neighbor_pos, goal);
                open_set.push(Node {
                    position: neighbor_pos,
                    g_cost: tentative_g,
                    h_cost: h,
                });
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::world::{Grid, Position};

    #[test]
    fn straight_line_path() {
        let goal = Position { x: 4, y: 0 };
        let grid = Grid::new(5, 1, goal);
        let start = (0, 0);

        let path = find_path(start, (4, 0), &grid).expect("path should exist");
        assert_eq!(path.first().copied(), Some(start));
        assert_eq!(path.last().copied(), Some((4, 0)));
        assert_eq!(path.len(), 5);
    }

    #[test]
    fn path_around_wall() {
        // 5x5 grid with a horizontal wall at y=2 except at x=2.
        let goal = Position { x: 4, y: 4 };
        let obstacles = [
            (0, 2),
            (1, 2),
            (3, 2),
            (4, 2),
        ];
        let grid = Grid::with_obstacles(5, 5, goal, &obstacles);

        let start = (0, 0);
        let path = find_path(start, (4, 4), &grid).expect("path should exist");
        assert_eq!(path.first().copied(), Some(start));
        assert_eq!(path.last().copied(), Some((4, 4)));
    }

    #[test]
    fn no_path_when_fully_blocked() {
        let goal = Position { x: 2, y: 2 };
        // Block all neighbors around start (0,0).
        let obstacles = [(1, 0), (0, 1)];
        let grid = Grid::with_obstacles(3, 3, goal, &obstacles);

        let start = (0, 0);
        let path = find_path(start, (2, 2), &grid);
        assert!(path.is_none());
    }
}
