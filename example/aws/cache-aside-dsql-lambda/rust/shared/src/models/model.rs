use rnglib::{Language, RNG};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, FromRow, Serialize, Deserialize, Clone)]
pub struct CacheableItem {
    pub id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl CacheableItem {
    fn new() -> CacheableItem {
        let rng = RNG::try_from(&Language::Elven).unwrap();
        CacheableItem {
            id: Uuid::new_v4(),
            first_name: rng.generate_name(),
            last_name: rng.generate_name(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }
}

impl Default for CacheableItem {
    fn default() -> Self {
        Self::new()
    }
}
