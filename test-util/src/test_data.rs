use momento::{
    cache::{Get, GetValue, SortedSetElements, SortedSetFetch},
    IntoBytes,
};
use uuid::Uuid;

pub fn unique_string(prefix: impl Into<String>) -> String {
    format!("{}-{}", prefix.into(), Uuid::new_v4())
}

pub fn unique_cache_name() -> String {
    unique_string("rust-sdk")
}

pub fn unique_key() -> String {
    unique_string("key")
}

pub fn unique_value() -> String {
    unique_string("value")
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

impl From<&TestScalar> for Get {
    fn from(test_scalar: &TestScalar) -> Self {
        Get::Hit {
            value: GetValue::new(test_scalar.value().into_bytes()),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct TestSet {
    pub name: String,
    pub elements: Vec<String>,
}

impl TestSet {
    pub fn new() -> Self {
        Self {
            name: unique_key(),
            elements: vec![unique_value(), unique_value()],
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn elements(&self) -> &Vec<String> {
        &self.elements
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
    pub elements: Vec<(String, f64)>,
}

impl TestSortedSet {
    pub fn new() -> Self {
        Self {
            name: unique_key(),
            elements: vec![(unique_value(), 1.0), (unique_value(), 2.0)],
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn elements(&self) -> &Vec<(String, f64)> {
        &self.elements
    }
}

impl Default for TestSortedSet {
    fn default() -> Self {
        Self::new()
    }
}

impl From<&TestSortedSet> for SortedSetFetch {
    fn from(test_sorted_set: &TestSortedSet) -> Self {
        SortedSetFetch::Hit {
            elements: SortedSetElements::new(
                test_sorted_set
                    .elements()
                    .iter()
                    .map(|(element, score)| (element.as_bytes().to_vec(), *score))
                    .collect(),
            ),
        }
    }
}
