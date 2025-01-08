use crate::leaderboard::MomentoRequest;
use crate::utils::prep_request_with_timeout;
use crate::{LeaderboardClient, MomentoResult};

use momento_protos::leaderboard::Element as ProtoElement;

/// This trait defines an interface for converting a type into a vector of [SortedSetElement].
pub trait IntoElements: Send {
    /// Converts the type into a vector of [SortedSetElement].
    fn into_elements(self) -> Vec<Element>;
}

#[cfg(not(doctest))]
pub fn map_and_collect_elements<I>(iter: I) -> Vec<Element>
where
    I: Iterator<Item = (u32, f64)>,
{
    iter.map(|(id, score)| Element { id, score }).collect()
}

impl IntoElements for Vec<(u32, f64)> {
    fn into_elements(self) -> Vec<Element> {
        map_and_collect_elements(self.into_iter())
    }
}

/// Represents an element in a sorted set.
/// Used by the various sorted set fetch methods to allow named access to value and score.
#[derive(Debug, PartialEq, Clone)]
pub struct Element {
    /// The value to be stored in the sorted set.
    pub id: u32,
    /// The score associated with the value.
    pub score: f64,
}

pub struct UpsertElementsRequest<E: IntoElements> {
    cache_name: String,
    leaderboard: String,
    elements: E,
}

impl<E: IntoElements> UpsertElementsRequest<E> {
    /// Constructs a new SortedSetPutElementsRequest.
    pub fn new(cache_name: impl Into<String>, leaderboard: impl Into<String>, elements: E) -> Self {
        Self {
            cache_name: cache_name.into(),
            leaderboard: leaderboard.into(),
            elements,
        }
    }
}

impl<E: IntoElements> MomentoRequest for UpsertElementsRequest<E> {
    type Response = UpsertElementsResponse;

    async fn send(self, leaderboard_client: &LeaderboardClient) -> MomentoResult<Self::Response> {
        let elements = self.elements.into_elements();
        let cache_name = self.cache_name.clone();
        let request = prep_request_with_timeout(
            &self.cache_name,
            leaderboard_client.deadline_millis(),
            momento_protos::leaderboard::UpsertElementsRequest {
                cache_name,
                leaderboard: self.leaderboard,
                elements: elements
                    .into_iter()
                    .map(|v| ProtoElement {
                        id: v.id,
                        score: v.score,
                    })
                    .collect(),
            },
        )?;

        let _ = leaderboard_client
            .next_data_client()
            .upsert_elements(request)
            .await?;
        Ok(Self::Response {})
    }
}

/// The response type for a successful `DeleteLeaderboardRequest`
#[derive(Debug, PartialEq, Eq)]
pub struct UpsertElementsResponse {}
