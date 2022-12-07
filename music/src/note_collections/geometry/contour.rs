

#[derive(Debug, Clone, PartialEq)]
pub enum Movement {
    Ascending,
    Descending,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Contour {
    Movement(Movement),
    Repeat,
}

#[derive(Debug, Clone, PartialEq)]
pub enum IntervalChange {
    Expanding,
    Contracting,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CompositeMovement {
    Parallel(Movement),
    Similar(Movement, IntervalChange),
    Oblique(Movement, IntervalChange),
    Contrary(IntervalChange),
    Repeat,
}