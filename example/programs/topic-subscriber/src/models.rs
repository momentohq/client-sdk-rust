use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct MomentoModel {
    pub key_one: String,
    pub key_two: String,
    pub key_three: i64,
    #[serde()]
    #[serde(rename(deserialize = "timestamp"))]
    pub published_timestamp: DateTime<Utc>,
}

impl MomentoModel {
    pub fn time_between_publish_and_received(&self) -> i64 {
        let received_time = Utc::now();
        received_time
            .signed_duration_since(self.published_timestamp)
            .num_milliseconds()
    }
}
