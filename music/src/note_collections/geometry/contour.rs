

pub enum Movement {
    Ascending,
    Descending,
}

pub enum Contour {
    Movement(Movement),
    Repeat,
}

pub enum IntervalChange {
    Expanding,
    Contracting,
}

pub enum CompositeMovement {
    Parallel(Movement),
    Similar(Movement, IntervalChange),
    Oblique(Movement, IntervalChange),
    Contrary(IntervalChange),
    Repeat,
}