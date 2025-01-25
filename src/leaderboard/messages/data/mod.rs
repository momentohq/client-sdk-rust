/// Contains the request and response types for deleting a leaderboard.
pub mod delete_leaderboard;
/// Contains the request and response types for requesting elements from a
/// leaderboard by rank.
pub mod get_by_rank;
/// Contains the request and response types for requesting elements from a
/// leaderboard by score.
pub mod get_by_score;
/// Contains the request and response types for getting the length of a
/// leaderboard.
pub mod get_leaderboard_length;
/// Contains the request and response types for requesting elements from a
/// leaderboard using their element ids.
pub mod get_rank;
/// Contains the request and response types for removing elements from a
/// leaderboard.
pub mod remove_elements;
/// Contains the request and response types for upserting (inserting/updating)
/// elements into a leaderboard.
pub mod upsert_elements;

// Common traits and enums

/// Represents an element in a leaderboard.
#[derive(Debug, PartialEq, Clone)]
pub struct RankedElement {
    /// The id of the element.
    pub id: u32,
    /// The rank of the element within the leaderboard.
    pub rank: u32,
    /// The score associated with the element.
    pub score: f64,
}

/// Specifies an ordering when requesting elements by rank or score.
#[repr(i32)]
pub enum Order {
    /// Elements will be ordered in ascending order.
    Ascending = 0,
    /// Elements will be ordered in descending order.
    Descending = 1,
}

/// This trait defines an interface for converting a type into a vector of
/// element ids.
pub trait IntoIds: Send {
    /// Converts the type into a vector of element ids.
    fn into_ids(self) -> Vec<u32>;
}

#[cfg(not(doctest))]
pub(crate) fn map_and_collect_elements<'a, I>(iter: I) -> Vec<u32>
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
