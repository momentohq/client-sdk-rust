use crate::leaderboard::MomentoRequest;
use crate::utils::prep_leaderboard_request_with_timeout;
use crate::{Leaderboard, MomentoResult};

use momento_protos::leaderboard::Element as ProtoElement;

/// This trait defines an interface for converting a type into a vector of [Element].
pub trait IntoElements: Send {
    /// Converts the type into a vector of [Element].
    fn into_elements(self) -> Vec<Element>;
}

/// Collects elements from an iterator into an owned collection.
#[cfg(not(doctest))]
pub(crate) fn map_and_collect_elements<I>(iter: I) -> Vec<Element>
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

/// Represents an element to be inserted into a leaderboard.
#[derive(Debug, PartialEq, Clone)]
pub struct Element {
    /// The id of the element.
    pub id: u32,
    /// The score associated with the element.
    pub score: f64,
}

/// A request to upsert (insert/update) elements into a leaderboard.
pub struct UpsertRequest<E: IntoElements> {
    elements: E,
}

impl<E: IntoElements> UpsertRequest<E> {
    /// Constructs a new `UpsertRequest`.
    pub fn new(elements: E) -> Self {
        Self { elements }
    }
}

impl<E: IntoElements> MomentoRequest for UpsertRequest<E> {
    type Response = UpsertResponse;

    async fn send(self, leaderboard: &Leaderboard) -> MomentoResult<Self::Response> {
        let elements = self.elements.into_elements();
        let cache_name = leaderboard.cache_name();
        let request = prep_leaderboard_request_with_timeout(
            cache_name,
            leaderboard.deadline(),
            momento_protos::leaderboard::UpsertElementsRequest {
                cache_name: cache_name.clone(),
                leaderboard: leaderboard.leaderboard_name().clone(),
                elements: elements
                    .into_iter()
                    .map(|v| ProtoElement {
                        id: v.id,
                        score: v.score,
                    })
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
