use web_sys::IdbCursorDirection;

/// Direction in which the key-value paris are fetched from the store.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Next,
    NextUnique,
    Prev,
    PrevUnique,
}

impl From<Direction> for IdbCursorDirection {
    fn from(direction: Direction) -> Self {
        match direction {
            Direction::Next => IdbCursorDirection::Next,
            Direction::NextUnique => IdbCursorDirection::Nextunique,
            Direction::Prev => IdbCursorDirection::Prev,
            Direction::PrevUnique => IdbCursorDirection::Prevunique,
        }
    }
}
