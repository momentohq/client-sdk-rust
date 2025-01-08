pub mod delete_leaderboard;
pub mod get_by_rank;
pub mod get_by_score;
pub mod get_leaderboard_length;
pub mod get_rank;
pub mod remove_elements;
pub mod upsert_elements;

// Common traits and enums

/// Represents an element in a leaderboard.
#[derive(Debug, PartialEq, Clone)]
pub struct RankedElement {
    /// The id of the element.
    pub id: u32,
    // The rank of the element within the leaderboard.
    pub rank: u32,
    /// The score associated with the element.
    pub score: f64,
}

#[repr(i32)]
pub enum Order {
    Ascending = 0,
    Descending = 1,
}

/// This trait defines an interface for converting a type into a vector of [SortedSetElement].
pub trait IntoIds: Send {
    /// Converts the type into a vector of [SortedSetElement].
    fn into_ids(self) -> Vec<u32>;
}

#[cfg(not(doctest))]
pub fn map_and_collect_elements<'a, I>(iter: I) -> Vec<u32>
where
    I: Iterator<Item = &'a u32>,
{
    iter.copied().collect()
}

impl IntoIds for Vec<u32> {
    fn into_ids(self) -> Vec<u32> {
        self
    }
}

impl IntoIds for &[u32] {
    fn into_ids(self) -> Vec<u32> {
        map_and_collect_elements(self.iter())
    }
}
