use crate::leaderboard::LeaderboardRequest;
use crate::utils::prep_leaderboard_request_with_timeout;
use crate::{Leaderboard, MomentoResult};

use momento_protos::leaderboard::Element as ProtoElement;

/// Represents an element to be inserted into a leaderboard.
#[derive(Debug, PartialEq, Clone)]
pub struct Element {
    /// The id of the element.
    pub id: u32,
    /// The score associated with the element.
    pub score: f64,
}

impl From<(u32, f64)> for Element {
    fn from((id, score): (u32, f64)) -> Self {
        Self { id, score }
    }
}

impl From<Element> for ProtoElement {
    fn from(element: Element) -> Self {
        Self {
            id: element.id,
            score: element.score,
        }
    }
}

/// A request to upsert (insert/update) elements into a leaderboard.
pub struct UpsertRequest<E, I>
where
    E: IntoIterator<Item = I> + Send,
    I: Into<Element>,
{
    elements: E,
}

impl<E, I> UpsertRequest<E, I>
where
    E: IntoIterator<Item = I> + Send,
    I: Into<Element>,
{
    /// Constructs a new `UpsertRequest`.
    pub fn new(elements: E) -> Self {
        Self { elements }
    }
}

impl<E, I> LeaderboardRequest for UpsertRequest<E, I>
where
    E: IntoIterator<Item = I> + Send,
    I: Into<Element>,
{
    type Response = UpsertResponse;

    async fn send(self, leaderboard: &Leaderboard) -> MomentoResult<Self::Response> {
        let cache_name = leaderboard.cache_name();
        let request = prep_leaderboard_request_with_timeout(
            cache_name,
            leaderboard.client_timeout(),
            momento_protos::leaderboard::UpsertElementsRequest {
                cache_name: cache_name.to_string(),
                leaderboard: leaderboard.leaderboard_name().to_string(),
                elements: self
                    .elements
                    .into_iter()
                    .map(Into::into)
                    .map(Into::into)
                    .collect(),
            },
        )?;

        let _ = leaderboard
            .next_data_client()
            .upsert_elements(request)
            .await?;
        Ok(Self::Response {})
    }
}

/// The response type for a successful `UpsertRequest`
#[derive(Debug, PartialEq, Eq)]
pub struct UpsertResponse {}
