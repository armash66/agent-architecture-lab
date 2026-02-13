use super::position::Position;

/// A simple 2D grid with a single goal cell.
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
}
