/// Contains the request and response types for deleting a leaderboard.
pub mod delete;
/// Contains the shared response type for fetching elements from a
/// leaderboard.
pub mod fetch;
/// Contains the request and response types for requesting elements from a
/// leaderboard by rank.
pub mod fetch_by_rank;
/// Contains the request and response types for requesting elements from a
/// leaderboard by score.
pub mod fetch_by_score;
pub mod get_competition_rank;
/// Contains the request and response types for requesting elements from a
/// leaderboard using their element ids.
pub mod get_rank;
/// Contains the request and response types for getting the length of a
/// leaderboard.
pub mod length;
/// Contains the request and response types for removing elements from a
/// leaderboard.
pub mod remove_elements;
/// Contains the request and response types for upserting (inserting/updating)
/// elements into a leaderboard.
pub mod upsert;

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

impl From<&momento_protos::leaderboard::RankedElement> for RankedElement {
    fn from(proto: &momento_protos::leaderboard::RankedElement) -> Self {
        Self {
            id: proto.id,
            rank: proto.rank,
            score: proto.score,
        }
    }
}

/// Specifies an ordering when requesting elements by rank or score.
pub enum Order {
    /// Elements will be ordered in ascending order.
    Ascending,
    /// Elements will be ordered in descending order.
    Descending,
}

impl From<Order> for momento_protos::leaderboard::Order {
    fn from(order: Order) -> Self {
        match order {
            Order::Ascending => momento_protos::leaderboard::Order::Ascending,
            Order::Descending => momento_protos::leaderboard::Order::Descending,
        }
    }
}

impl Order {
    /// Converts the order into a proto enum.
    pub fn into_proto(self) -> momento_protos::leaderboard::Order {
        self.into()
    }
}
