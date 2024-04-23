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

    pub fn key(&self) -> String {
        self.key.clone()
    }

    pub fn value(&self) -> String {
        self.value.clone()
    }
}

impl Default for TestScalar {
    fn default() -> Self {
        Self::new()
    }
}

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

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn elements(&self) -> Vec<String> {
        self.elements.clone()
    }
}

impl Default for TestSet {
    fn default() -> Self {
        Self::new()
    }
}

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

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn elements(&self) -> Vec<(String, f64)> {
        self.elements.clone()
    }
}

impl Default for TestSortedSet {
    fn default() -> Self {
        Self::new()
    }
}
