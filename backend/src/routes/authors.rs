use crate::{error::AppError, extractors::SiteIdentity, models::{Author, AuthorSocial}};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Serialize)]
pub struct AuthorResponse {
    pub uuid: Uuid,
    pub name: String,
    pub bio: Option<String>,
    pub signing_email: Option<String>,
    pub socials: Vec<SocialResponse>,
}

#[derive(Serialize)]
pub struct SocialResponse {
    pub platform: String,
    pub handle: String,
    pub url: Option<String>,
    pub visibility_mask: i32,
}

pub async fn get_authors(
    State(pool): State<PgPool>,
    site: SiteIdentity,
) -> Result<Json<Vec<AuthorResponse>>, AppError> {
    let authors = sqlx::query_as::<_, Author>("SELECT id, uuid, name, bio, signing_email FROM authors")
        .fetch_all(&pool)
        .await?;

    let mut response = Vec::new();
    for author in authors {
        let socials = sqlx::query_as::<_, AuthorSocial>(
            "SELECT id, author_uuid, platform, handle, url, visibility_mask FROM author_socials WHERE author_uuid = $1 AND (visibility_mask & $2) > 0"
        )
        .bind(author.uuid)
        .bind(site.mask)
        .fetch_all(&pool)
        .await?
        .into_iter()
        .map(|s| SocialResponse {
            platform: s.platform,
            handle: s.handle,
            url: s.url,
            visibility_mask: s.visibility_mask,
        })
        .collect();

        response.push(AuthorResponse {
            uuid: author.uuid,
            name: author.name,
            bio: author.bio,
            signing_email: author.signing_email,
            socials,
        });
    }

    Ok(Json(response))
}

#[derive(Deserialize)]
pub struct CreateAuthorRequest {
    pub name: String,
    pub bio: Option<String>,
    pub signing_email: Option<String>,
}

pub async fn create_author(
    State(pool): State<PgPool>,
    site: SiteIdentity,
    Json(payload): Json<CreateAuthorRequest>,
) -> Result<Json<AuthorResponse>, AppError> {
    if !site.requires_auth {
        return Err(AppError::unauthorized().at_site(&site));
    }

    let author = sqlx::query_as::<_, Author>(
        "INSERT INTO authors (name, bio, signing_email) VALUES ($1, $2, $3) RETURNING id, uuid, name, bio, signing_email"
    )
    .bind(&payload.name)
    .bind(&payload.bio)
    .bind(&payload.signing_email)
    .fetch_one(&pool)
    .await?;

    Ok(Json(AuthorResponse {
        uuid: author.uuid,
        name: author.name,
        bio: author.bio,
        signing_email: author.signing_email,
        socials: Vec::new(),
    }))
}

#[derive(Deserialize)]
pub struct AddSocialRequest {
    pub platform: String,
    pub handle: String,
    pub url: Option<String>,
    pub visibility_mask: i32,
}

pub async fn add_social(
    State(pool): State<PgPool>,
    site: SiteIdentity,
    Path(author_uuid): Path<Uuid>,
    Json(payload): Json<AddSocialRequest>,
) -> Result<StatusCode, AppError> {
    if !site.requires_auth {
        return Err(AppError::unauthorized().at_site(&site));
    }

    sqlx::query(
        "INSERT INTO author_socials (author_uuid, platform, handle, url, visibility_mask) VALUES ($1, $2, $3, $4, $5)"
    )
    .bind(author_uuid)
    .bind(&payload.platform)
    .bind(&payload.handle)
    .bind(&payload.url)
    .bind(payload.visibility_mask)
    .execute(&pool)
    .await?;

    Ok(StatusCode::CREATED)
}
