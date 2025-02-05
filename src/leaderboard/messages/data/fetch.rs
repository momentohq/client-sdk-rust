use super::RankedElement;

/// The response type for a successful `FetchByRankRequest` or
/// `FetchByScoreRequest`.
#[derive(Debug, PartialEq)]
pub struct FetchResponse {
    elements: Vec<RankedElement>,
}

impl FetchResponse {
    /// Constructs a new `FetchResponse`.
    pub fn new(elements: Vec<RankedElement>) -> Self {
        Self { elements }
    }

    /// Returns the ranked elements in the response.
    pub fn elements(&self) -> &[RankedElement] {
        &self.elements
    }

    /// Consumes the `FetchResponse` and returns the ranked elements.
    pub fn into_elements(self) -> Vec<RankedElement> {
        self.elements
    }
}
