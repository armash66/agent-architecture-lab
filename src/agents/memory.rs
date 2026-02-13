use std::collections::VecDeque;

use crate::engine::world::Position;

/// A bounded ring-buffer of visited positions.
///
/// When the capacity is reached, the oldest entry is evicted.
/// A capacity of 0 means memory is disabled (never remembers anything).
pub struct SpatialMemory {
    capacity: usize,
    entries: VecDeque<Position>,
}

impl SpatialMemory {
    /// Create a new memory with the given capacity.
    /// Capacity 0 disables memory entirely.
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            entries: VecDeque::with_capacity(capacity.min(256)),
        }
    }

    /// Record a position. Evicts the oldest entry if at capacity.
    pub fn record(&mut self, pos: Position) {
        if self.capacity == 0 {
            return;
        }
        // Don't record duplicates of the most recent entry.
        if self.entries.back() == Some(&pos) {
            return;
        }
        if self.entries.len() >= self.capacity {
            self.entries.pop_front();
        }
        self.entries.push_back(pos);
    }

    /// Check whether a position has been visited recently (is in memory).
    pub fn contains(&self, pos: &Position) -> bool {
        self.entries.contains(pos)
    }

    /// Number of entries currently stored.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether memory is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// The capacity of this memory.
    pub fn capacity(&self) -> usize {
        self.capacity
    }
}
