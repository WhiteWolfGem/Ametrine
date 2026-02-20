use crate::{error::AppError, extractors::SiteIdentity, models::Post, params::SearchParams};
use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

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
    updated: DateTime<Utc>,
    tags: serde_json::Value,
    signature: Option<String>,
    is_mature: bool,
    summary: Option<String>,
}

impl From<Post> for PostResponse {
    fn from(post: Post) -> Self {
        Self {
            uuid: post.uuid,
            slug: post.slug,
            title: post.title,
            content: post.content,
            created: post.created_at,
            updated: post.updated_at,
            tags: post.tags,
            signature: post.signature,
            is_mature: post.is_mature,
            summary: post.summary,
        }
    }
}

pub async fn get_one_post(
    State(pool): State<PgPool>,
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
                updated_at,
                tags,
                signature,
                is_mature,
                summary
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
                updated_at,
                tags,
                signature,
                is_mature,
                summary
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
    State(pool): State<PgPool>,
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
            updated_at,
            tags,
            signature,
            is_mature,
            summary
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
    pub signature: Option<String>,
    pub is_mature: bool,
    pub summary: Option<String>,
}

pub async fn create_post(
    State(pool): State<PgPool>,
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
            INSERT INTO posts (
                uuid, 
                title, 
                slug, 
                content, 
                tags, 
                visibility_mask,
                signature,
                is_mature,
                summary
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING 
            id,
            uuid,
            title,
            slug,
            content,
            created_at,
            updated_at,
            tags,
            signature,
            is_mature,
            summary
        "#,
    )
    .bind(new_uuid)
    .bind(&payload.title)
    .bind(&payload.slug)
    .bind(&payload.content)
    .bind(&tags_json)
    .bind(payload.visibility_mask)
    .bind(&payload.signature)
    .bind(payload.is_mature)
    .bind(&payload.summary)
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

pub async fn delete_post(
    State(pool): State<PgPool>,
    Path(uuid): Path<uuid::Uuid>,
) -> Result<axum::http::StatusCode, AppError> {
    let mut tx = pool.begin().await?;

    sqlx::query!(
        r#"
        UPDATE tag_stats
        SET use_count = use_count - 1
        WHERE tag_name IN (
            SELECT jsonb_array_elements_text(tags) FROM posts where uuid = $1
        )
        "#,
        uuid
    )
    .execute(&mut *tx)
    .await?;

    sqlx::query!(
        r#"
        DELETE FROM posts
        WHERE uuid = $1
        
        "#,
        uuid
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(StatusCode::NO_CONTENT)
}
#[derive(serde::Deserialize)]
pub struct UpdatePostRequest {
    pub title: String,
    pub slug: Option<String>,
    pub content: String,
    pub tags: Vec<String>,
    pub visibility_mask: i32,
    pub signature: Option<String>,
    pub is_mature: bool,
    pub summary: Option<String>,
}

pub async fn update_post(
    State(pool): State<PgPool>,
    site: SiteIdentity,
    Path(uuid): Path<uuid::Uuid>,
    Json(payload): Json<UpdatePostRequest>,
) -> Result<Json<PostResponse>, AppError> {
    if !site.requires_auth {
        return Err(AppError::Unauthorized);
    }
    if !site.domain.starts_with("localhost") && !site.domain.starts_with("127.0.0.1") {
        return Err(AppError::Unauthorized);
    }

    let mut tx = pool.begin().await?;

    let old_post = sqlx::query!("SELECT tags FROM posts WHERE uuid = $1", uuid)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or(AppError::NotFound)?;

    let old_tags: Vec<String> = serde_json::from_value(old_post.tags.unwrap_or_default())
        .map_err(|_| AppError::Anyhow(anyhow::anyhow!("Failed to parse old tags")))?;

    let old_tags_set: std::collections::HashSet<_> = old_tags.iter().collect();
    let new_tags_set: std::collections::HashSet<_> = payload.tags.iter().collect();

    let tags_to_remove: Vec<_> = old_tags_set.difference(&new_tags_set).collect();
    let tags_to_add: Vec<_> = new_tags_set.difference(&old_tags_set).collect();

    for tag_name in tags_to_remove {
        sqlx::query!(
            r#"
                UPDATE tag_stats SET use_count = use_count - 1 where tag_name = $1
            "#,
            tag_name
        )
        .execute(&mut *tx)
        .await?;
    }
    for tag_name in tags_to_add {
        let tag_uuid = uuid::Uuid::new_v4();
        sqlx::query!(
            r#"
                INSERT INTO tag_stats (tag_uuid, tag_name, visibility_mask, use_count)
                VALUES ($1, $2, $3, 1)
                ON CONFLICT (tag_name)
                DO UPDATE SET
                    use_count = tag_stats.use_count + 1,
                    visibility_mask = tag_stats.visibility_mask | EXCLUDED.visibility_mask
            "#,
            tag_uuid,
            tag_name,
            payload.visibility_mask
        )
        .execute(&mut *tx)
        .await?;
    }

    let tags_json = serde_json::to_value(&payload.tags)
        .map_err(|_| AppError::Anyhow(anyhow::anyhow!("Failed to parse tags")))?;

    let post = sqlx::query_as::<_, Post>(
        r#"
                UPDATE 
                    posts 
                SET 
                    title = $1,
                    slug = $2,
                    content = $3,
                    tags = $4,
                    visibility_mask = $5,
                    signature = $6,
                    is_mature = $7,
                    summary = $8,
                    updated_at = $9
                WHERE 
                    uuid = $10
                RETURNING 
                    id,
                    uuid,
                    title,
                    slug,
                    content,
                    created_at,
                    updated_at,
                    tags,
                    signature,
                    is_mature,
                    summary
            "#,
    )
    .bind(&payload.title)
    .bind(&payload.slug)
    .bind(&payload.content)
    .bind(&tags_json)
    .bind(payload.visibility_mask)
    .bind(&payload.signature)
    .bind(payload.is_mature)
    .bind(&payload.summary)
    .bind(Utc::now())
    .bind(uuid)
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(Json(post.into()))
}
