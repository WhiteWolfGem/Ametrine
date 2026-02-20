use crate::{error::AppError, extractors::SiteIdentity, models::Post, params::SearchParams};
use axum::{
    Json,
    extract::{Path, Query, State},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum PostSort {
    Date,
    Title,
}

#[derive(serde::Deserialize)]
pub struct PostParams {
    #[serde(flatten)]
    base: SearchParams<PostSort>,
    tag: Option<String>,
}

impl PostParams {}
#[derive(Serialize)]
pub struct PostResponse {
    uuid: uuid::Uuid,
    slug: Option<String>,
    title: String,
    content: String,
    created: DateTime<Utc>,
    tags: serde_json::Value,
}

impl From<Post> for PostResponse {
    fn from(post: Post) -> Self {
        Self {
            uuid: post.uuid,
            slug: post.slug,
            title: post.title,
            content: post.content,
            created: post.created_at,
            tags: post.tags,
        }
    }
}

pub async fn get_one_post(
    State(pool): State<sqlx::PgPool>,
    site: SiteIdentity,
    Path(identifier): Path<String>,
) -> Result<Json<PostResponse>, AppError> {
    let query = if let Ok(id) = uuid::Uuid::parse_str(&identifier) {
        sqlx::query_as::<_, Post>(
            "SELECT 
                id, 
                uuid,
                title, 
                slug,
                content, 
                created_at, 
                tags 
            FROM 
                posts
            WHERE
                uuid = $1
            AND 
                (visibility_mask & $2) > 0
            ",
        )
        .bind(id)
        .bind(site.mask)
    } else {
        sqlx::query_as::<_, Post>(
            "SELECT 
                id, 
                uuid,
                title, 
                slug,
                content, 
                created_at, 
                tags 
            FROM 
                posts
            WHERE
                slug = $1
            AND 
                (visibility_mask & $2) > 0
            ",
        )
        .bind(identifier)
        .bind(site.mask)
    };
    let post = query
        .fetch_optional(&pool)
        .await?
        .ok_or(AppError::NotFound)?;

    Ok(Json(post.into()))
}
pub async fn get_posts(
    State(pool): State<sqlx::PgPool>,
    site: SiteIdentity,
    Query(params): Query<PostParams>,
) -> Result<Json<Vec<PostResponse>>, AppError> {
    let limit = params.base.limit();
    let offset = params.base.offset();
    let column = match params.base.sort() {
        Some(PostSort::Title) => "REGEXP_REPLACE(title, '^(The|A|An)\\s+', '', 'i')",
        _ => "created_at",
    };

    let direction = params.base.sort_by().to_sql();

    let search_pattern = params.base.search().map(|s| format!("%{}%", s));

    let query = format!(
        r#"SELECT 
            id, 
            uuid,
            title, 
            slug,
            content, 
            created_at, 
            tags 
        FROM 
            posts
        WHERE
            (visibility_mask & $1) > 0
        AND
            ($4::TEXT IS NULL or tags ? $4)
        AND
            ($5::TEXT is NULL or title ILIKE $5)
        ORDER BY 
            {} {}
        LIMIT $2 OFFSET $3"#,
        column, direction
    );
    let posts = sqlx::query_as::<_, Post>(&query)
        .bind(site.mask)
        .bind(limit)
        .bind(offset)
        .bind(&params.tag)
        .bind(search_pattern)
        .fetch_all(&pool)
        .await?;

    let response: Vec<PostResponse> = posts.into_iter().map(|p| p.into()).collect();

    Ok(Json(response))
}

#[derive(serde::Deserialize)]
pub struct CreatePostRequest {
    pub title: String,
    pub slug: Option<String>,
    pub content: String,
    pub tags: Vec<String>,
    pub visibility_mask: i32,
}

pub async fn create_post(
    State(pool): State<sqlx::PgPool>,
    site: SiteIdentity,
    axum::Json(payload): axum::Json<CreatePostRequest>,
) -> Result<Json<PostResponse>, AppError> {
    if !site.requires_auth {
        return Err(AppError::Unauthorized);
    }

    if !site.domain.starts_with("localhost") && !site.domain.starts_with("127.0.0.1") {
        // TODO: JWT/Oauth stuff here
        return Err(AppError::Unauthorized);
    }

    let new_uuid = uuid::Uuid::new_v4();
    let tags_json = serde_json::to_value(&payload.tags)
        .map_err(|_| AppError::Anyhow(anyhow::anyhow!("Failed to parse tags")))?;

    let mut tx = pool.begin().await?;

    let post = sqlx::query_as::<_, Post>(
        r#"
            INSERT INTO posts (uuid, title, slug, content, tags, visibility_mask)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id,uuid,title,slug,content, created_at,tags
        "#,
    )
    .bind(new_uuid)
    .bind(&payload.title)
    .bind(&payload.slug)
    .bind(&payload.content)
    .bind(&tags_json)
    .bind(payload.visibility_mask)
    .fetch_one(&mut *tx)
    .await?;

    for tag_name in &payload.tags {
        let tag_uuid = uuid::Uuid::new_v4();

        sqlx::query!(
            r#"
            INSERT INTO tag_stats (tag_uuid, tag_name, visibility_mask, use_count)
            VALUES ($1,$2,$3,1)
            ON CONFLICT (tag_name)
            DO UPDATE SET
                use_count = tag_stats.use_count +1,
                visibility_mask = tag_stats.visibility_mask | EXCLUDED.visibility_mask
            "#,
            tag_uuid,
            tag_name,
            payload.visibility_mask
        )
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    Ok(Json(post.into()))
}
