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
    pub updated_at: DateTime<Utc>,
    pub tags: serde_json::Value,
    pub signature: Option<String>,
    pub is_mature: bool,
    pub summary: Option<String>,
    pub author_uuid: Option<Uuid>,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct Tag {
    pub tag_name: String,
    pub tag_uuid: Uuid,
    pub use_count: i32,
    pub selected_count: i32,
    pub visibility_mask: i32,
}

#[derive(Serialize, Deserialize, sqlx::FromRow)]
pub struct Author {
    pub id: i32,
    pub uuid: Uuid,
    pub name: String,
    pub bio: Option<String>,
    pub signing_email: Option<String>,
}

#[derive(Serialize, Deserialize, sqlx::FromRow)]
pub struct AuthorSocial {
    pub id: i32,
    pub author_uuid: Uuid,
    pub platform: String,
    pub handle: String,
    pub url: Option<String>,
    pub visibility_mask: i32,
}
