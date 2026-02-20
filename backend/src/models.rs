use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, sqlx::FromRow)]
pub struct Post {
    pub id: i32,
    pub uuid: Uuid,
    pub slug: Option<String>,
    pub title: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub tags: serde_json::Value,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct Tag {
    pub tag_name: String,
    pub tag_uuid: Uuid,
    pub use_count: i32,
    pub selected_count: i32,
    pub visibility_mask: i32,
}
