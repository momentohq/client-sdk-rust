use momento::{
    cache::{GetResponse, ListFetchResponse, SortedSetElements, SortedSetFetchResponse},
    leaderboard::{messages::data::fetch::FetchResponse, Element, RankedElement},
};
use std::collections::HashMap;
use uuid::Uuid;

pub fn unique_string(prefix: impl Into<String>) -> String {
    format!("{}-{}", prefix.into(), Uuid::new_v4())
}

pub fn unique_cache_name() -> String {
    unique_string("rust-sdk")
}

pub fn unique_leaderboard_name() -> String {
    unique_string("leaderboard")
}

pub fn unique_key() -> String {
    unique_string("key")
}

pub fn unique_value() -> String {
    unique_string("value")
}

pub fn unique_topic_name() -> String {
    unique_string("topic")
}

pub fn unique_store_name() -> String {
    unique_string("rust-sdk")
}

#[derive(Debug, PartialEq, Clone)]
pub struct TestScalar {
    pub key: String,
    pub value: String,
}

impl TestScalar {
    pub fn new() -> Self {
        Self {
            key: unique_key(),
            value: unique_value(),
        }
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn value(&self) -> &str {
        &self.value
    }
}

impl Default for TestScalar {
    fn default() -> Self {
        Self::new()
    }
}

impl From<&TestScalar> for GetResponse {
    fn from(test_scalar: &TestScalar) -> Self {
        test_scalar.value().into()
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct TestDictionary {
    pub name: String,
    pub value: HashMap<String, String>,
}

impl TestDictionary {
    pub fn new() -> Self {
        Self {
            name: unique_key(),
            value: vec![
                (unique_key(), unique_value()),
                (unique_key(), unique_value()),
            ]
            .into_iter()
            .collect(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn value(&self) -> &HashMap<String, String> {
        &self.value
    }
}

impl Default for TestDictionary {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct TestSet {
    pub name: String,
    pub value: Vec<String>,
}

impl TestSet {
    pub fn new() -> Self {
        Self {
            name: unique_key(),
            value: vec![unique_value(), unique_value()],
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn value(&self) -> &Vec<String> {
        &self.value
    }
}

impl Default for TestSet {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct TestSortedSet {
    pub name: String,
    pub value: Vec<(String, f64)>,
}

impl TestSortedSet {
    pub fn new() -> Self {
        Self {
            name: unique_key(),
            value: vec![(unique_value(), 1.0), (unique_value(), 2.0)],
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn value(&self) -> &Vec<(String, f64)> {
        &self.value
    }
}

impl Default for TestSortedSet {
    fn default() -> Self {
        Self::new()
    }
}

impl From<&TestSortedSet> for SortedSetFetchResponse {
    fn from(test_sorted_set: &TestSortedSet) -> Self {
        SortedSetFetchResponse::Hit {
            value: SortedSetElements::new(
                test_sorted_set
                    .value()
                    .iter()
                    .map(|(element, score)| (element.as_bytes().to_vec(), *score))
                    .collect(),
            ),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct TestList {
    pub name: String,
    pub values: Vec<String>,
}

impl TestList {
    pub fn new() -> Self {
        Self {
            name: unique_key(),
            values: vec![unique_value(), unique_value()],
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn values(&self) -> &Vec<String> {
        &self.values
    }
}

impl Default for TestList {
    fn default() -> Self {
        Self::new()
    }
}

impl From<&TestList> for ListFetchResponse {
    fn from(test_list: &TestList) -> Self {
        test_list.values().clone().into()
    }
}

impl From<TestList> for ListFetchResponse {
    fn from(test_list: TestList) -> Self {
        test_list.values().clone().into()
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct TestLeaderboard {
    pub ranked_elements: Vec<RankedElement>,
}

impl TestLeaderboard {
    pub fn new() -> Self {
        Self {
            ranked_elements: vec![
                RankedElement {
                    id: 0,
                    rank: 0,
                    score: 1.0,
                },
                RankedElement {
                    id: 1,
                    rank: 1,
                    score: 2.0,
                },
            ],
        }
    }

    pub fn ranked_elements(&self) -> Vec<RankedElement> {
        self.ranked_elements.clone()
    }

    pub fn elements(&self) -> Vec<Element> {
        self.ranked_elements
            .iter()
            .map(|element| Element {
                id: element.id,
                score: element.score,
            })
            .collect()
    }

    pub fn ids(&self) -> Vec<u32> {
        self.ranked_elements
            .iter()
            .map(|element| element.id)
            .collect()
    }
}

impl Default for TestLeaderboard {
    fn default() -> Self {
        Self::new()
    }
}

impl From<&TestLeaderboard> for FetchResponse {
    fn from(test_leaderboard: &TestLeaderboard) -> Self {
        FetchResponse::new(test_leaderboard.ranked_elements().clone())
    }
}
