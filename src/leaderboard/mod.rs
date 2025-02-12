/// Containts the request and response types for leaderboard operations.
pub mod messages;

pub use messages::LeaderboardRequest;

pub use messages::data::delete::{DeleteRequest, DeleteResponse};
pub use messages::data::fetch_by_rank::{FetchByRankRequest, RankRange};
pub use messages::data::fetch_by_score::{FetchByScoreRequest, ScoreRange};
pub use messages::data::get_rank::GetRankRequest;
pub use messages::data::length::{LengthRequest, LengthResponse};
pub use messages::data::remove_elements::{RemoveElementsRequest, RemoveElementsResponse};
pub use messages::data::upsert::{Element, UpsertRequest, UpsertResponse};
pub use messages::data::{Order, RankedElement};

mod config;
mod leaderboard_client;
mod leaderboard_client_builder;
mod leaderboard_resource;

pub use config::configuration::Configuration;
pub use config::configurations;

pub use leaderboard_client::LeaderboardClient;
pub use leaderboard_resource::Leaderboard;
