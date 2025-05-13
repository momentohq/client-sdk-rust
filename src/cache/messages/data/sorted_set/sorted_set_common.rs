/// Boundary for a sorted set score range.
#[derive(Debug, PartialEq, Clone)]
pub enum ScoreBound {
    /// Include the score in the range.
    Inclusive(f64),
    /// Exclude the score from the range.
    Exclusive(f64),
}

/// The order with which to sort the elements by score in the sorted set.
/// The sort order determines the rank of the elements.
/// The elements with same score are ordered lexicographically.
#[repr(i32)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum SortedSetOrder {
    /// Scores are ordered from low to high. This is the default order.
    Ascending = 0,
    /// Scores are ordered from high to low.
    Descending = 1,
}
