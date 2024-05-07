use std::collections::HashMap;
use std::marker::PhantomData;

use momento_protos::cache_client::SortedSetElement as ProtoSortedSetElement;
use momento_protos::cache_client::SortedSetPutRequest;

use crate::cache::messages::MomentoRequest;
use crate::cache::CollectionTtl;
use crate::utils::prep_request_with_timeout;
use crate::{CacheClient, IntoBytes, MomentoResult};

/// This trait defines an interface for converting a type into a vector of [SortedSetElement].
pub trait IntoSortedSetElements<V: IntoBytes>: Send {
    /// Converts the type into a vector of [SortedSetElement].
    fn into_sorted_set_elements(self) -> Vec<SortedSetElement<V>>;
}

// This should be used by the various sorted set fetch methods.
// That way we have named access to value and score.
#[derive(Debug, PartialEq, Clone)]
pub struct SortedSetElement<V: IntoBytes> {
    pub value: V,
    pub score: f64,
}

/// Converts an iterator of value-score pairs into a vector of [SortedSetElement]s.
///
/// # Arguments
///
/// - `iter`: An iterator over pairs `(V, f64)` where `V` is a value that implements
///   the `IntoBytes` trait, and `f64` represents the score associated with the value.
///
/// # Returns
///
/// A `Vec<SortedSetElement>` where each `SortedSetElement` contains a byte representation
/// of the value and its associated score. The order of elements in the returned vector
/// matches the order of pairs in the input iterator.
///
/// # Examples
///
/// Basic usage with a vector of tuples:
///
/// ```
/// let pairs = vec![("value1", 1.0), ("value2", 2.0)];
/// let sorted_set_elements = map_and_collect_sorted_set_elements(pairs.into_iter());
/// // `sorted_set_elements` is now a `Vec<SortedSetElement>` with byte representations of "value1" and "value2"
/// # assert_eq!(sorted_set_elements, vec![
/// #    SortedSetElement {
/// #        value: "value1",
/// #        score: 1.0,
/// #    },
/// #    SortedSetElement {
/// #        value: "value2",
/// #        score: 2.0,
/// #    },
/// # ]);
/// ```
///
/// Usage with a `HashMap`:
///
/// ```
/// use std::collections::HashMap;
/// let mut map = HashMap::new();
/// map.insert("value1", 1.0);
/// map.insert("value2", 2.0);
/// let sorted_set_elements = map_and_collect_sorted_set_elements(map.into_iter());
/// // `sorted_set_elements` is similar as above, suitable for sorted set operations
/// # assert_eq!(sorted_set_elements, vec![
/// #    SortedSetElement {
/// #        value: "value1",
/// #        score: 1.0,
/// #    },
/// #    SortedSetElement {
/// #        value: "value2",
/// #        score: 2.0,
/// #    },
/// # ]);
/// ```
#[cfg(not(doctest))]
fn map_and_collect_sorted_set_elements<I, V>(iter: I) -> Vec<SortedSetElement<V>>
where
    I: Iterator<Item = (V, f64)>,
    V: IntoBytes,
{
    iter.map(|(value, score)| SortedSetElement { value, score })
        .collect()
}

impl<V: IntoBytes> IntoSortedSetElements<V> for Vec<(V, f64)> {
    fn into_sorted_set_elements(self) -> Vec<SortedSetElement<V>> {
        map_and_collect_sorted_set_elements(self.into_iter())
    }
}

impl<V: IntoBytes> IntoSortedSetElements<V> for Vec<SortedSetElement<V>> {
    fn into_sorted_set_elements(self) -> Vec<SortedSetElement<V>> {
        self
    }
}

impl<V: IntoBytes> IntoSortedSetElements<V> for HashMap<V, f64> {
    fn into_sorted_set_elements(self) -> Vec<SortedSetElement<V>> {
        map_and_collect_sorted_set_elements(self.into_iter())
    }
}

/// Request to add elements to a sorted set. If an element already exists, its score is updated.
/// Creates the sorted set if it does not exist.
///
/// # Arguments
///
/// * `cache_name` - The name of the cache containing the sorted set.
/// * `sorted_set_name` - The name of the sorted set ot add an element to.
/// * `elements` - The values and scores to add.
///
/// # Optional Arguments
///
/// * `collection_ttl` - The time-to-live for the collection. If not provided, the client's default time-to-live is used.
///
/// # Examples
/// Assumes that a CacheClient named `cache_client` has been created and is available.
/// ```
/// # fn main() -> anyhow::Result<()> {
/// # use momento_test_util::create_doctest_cache_client;
/// # tokio_test::block_on(async {
/// use momento::cache::{CollectionTtl, SortedSetPutElements, SortedSetPutElementsRequest};
/// # let (cache_client, cache_name) = create_doctest_cache_client();
/// let sorted_set_name = "sorted_set";
///
/// let put_elements_request = SortedSetPutElementsRequest::new(
///     cache_name,
///     sorted_set_name,
///     vec![("value1", 1.0), ("value2", 2.0)]
/// ).ttl(CollectionTtl::default());
///
/// match cache_client.send_request(put_elements_request).await {
///     Ok(_) => println!("Elements added to sorted set"),
///     Err(e) => eprintln!("Error adding elements to sorted set: {}", e),
/// }
/// # Ok(())
/// # })
/// # }
/// ```
pub struct SortedSetPutElementsRequest<S: IntoBytes, V: IntoBytes, E: IntoSortedSetElements<V>> {
    cache_name: String,
    sorted_set_name: S,
    elements: E,
    collection_ttl: Option<CollectionTtl>,
    // V is only used for the `IntoSortedSetElement`'s generic type parameter.
    _marker: PhantomData<V>,
}

impl<S: IntoBytes, V: IntoBytes, E: IntoSortedSetElements<V>> SortedSetPutElementsRequest<S, V, E> {
    pub fn new(cache_name: impl Into<String>, sorted_set_name: S, elements: E) -> Self {
        let collection_ttl = CollectionTtl::default();
        Self {
            cache_name: cache_name.into(),
            sorted_set_name,
            elements,
            collection_ttl: Some(collection_ttl),
            _marker: PhantomData,
        }
    }

    /// Set the time-to-live for the collection.
    pub fn ttl(mut self, collection_ttl: CollectionTtl) -> Self {
        self.collection_ttl = Some(collection_ttl);
        self
    }
}

impl<S: IntoBytes, V: IntoBytes, E: IntoSortedSetElements<V>> MomentoRequest
    for SortedSetPutElementsRequest<S, V, E>
{
    type Response = SortedSetPutElements;

    async fn send(self, cache_client: &CacheClient) -> MomentoResult<SortedSetPutElements> {
        let collection_ttl = self.collection_ttl.unwrap_or_default();
        let elements = self.elements.into_sorted_set_elements();
        let set_name = self.sorted_set_name.into_bytes();
        let cache_name = &self.cache_name;
        let request = prep_request_with_timeout(
            cache_name,
            cache_client.configuration.deadline_millis(),
            SortedSetPutRequest {
                set_name,
                elements: elements
                    .into_iter()
                    .map(|element| ProtoSortedSetElement {
                        value: element.value.into_bytes(),
                        score: element.score,
                    })
                    .collect(),
                ttl_milliseconds: cache_client.expand_ttl_ms(collection_ttl.ttl())?,
                refresh_ttl: collection_ttl.refresh(),
            },
        )?;

        let _ = cache_client
            .data_client
            .clone()
            .sorted_set_put(request)
            .await?;
        Ok(SortedSetPutElements {})
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct SortedSetPutElements {}
