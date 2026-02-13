use super::position::Position;

/// A simple 2D grid with a single goal cell.
#[derive(Clone)]
pub struct Grid {
    pub width: usize,
    pub height: usize,
    pub goal: Position,
    tiles: Vec<Vec<bool>>,
}

impl Grid {
    pub fn new(width: usize, height: usize, goal: Position) -> Self {
        let tiles = vec![vec![true; width]; height];

        Self {
            width,
            height,
            goal,
            tiles,
        }
    }

    /// Convenience constructor for tests or experiments that need obstacles.
    /// `obstacles` is a list of (x, y) cells that are *not* walkable.
    pub fn with_obstacles(
        width: usize,
        height: usize,
        goal: Position,
        obstacles: &[(usize, usize)],
    ) -> Self {
        let mut grid = Self::new(width, height, goal);

        for &(x, y) in obstacles {
            if x < width && y < height && (x != goal.x || y != goal.y) {
                grid.tiles[y][x] = false;
            }
        }

        grid
    }

    /// Return whether the given cell is walkable (in-bounds and not blocked).
    pub fn is_walkable(&self, x: usize, y: usize) -> bool {
        if x >= self.width || y >= self.height {
            return false;
        }
        self.tiles[y][x]
    }

    /// Return a random walkable neighbor of `(x, y)`, or `None` if boxed in.
    pub fn random_walkable_neighbor(&self, x: usize, y: usize) -> Option<(usize, usize)> {
        use rand::seq::SliceRandom;

        let mut candidates = Vec::new();
        if x > 0 && self.is_walkable(x - 1, y) {
            candidates.push((x - 1, y));
        }
        if x + 1 < self.width && self.is_walkable(x + 1, y) {
            candidates.push((x + 1, y));
        }
        if y > 0 && self.is_walkable(x, y - 1) {
            candidates.push((x, y - 1));
        }
        if y + 1 < self.height && self.is_walkable(x, y + 1) {
            candidates.push((x, y + 1));
        }

        let mut rng = rand::thread_rng();
        candidates.choose(&mut rng).copied()
    }
}
