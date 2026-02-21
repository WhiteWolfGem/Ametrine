use crate::{error::AppError, extractors::SiteIdentity};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

#[derive(Serialize)]
pub struct SiteResponse {
    pub id: i32,
    pub domain: String,
    pub site_mask_bit: i32,
    pub requires_auth: bool,
}

pub async fn get_sites(
    State(pool): State<PgPool>,
    site: SiteIdentity,
) -> Result<Json<Vec<SiteResponse>>, AppError> {
    // Only allow localhost to manage sites
    if !site.domain.starts_with("localhost") && !site.domain.starts_with("127.0.0.1") {
        return Err(AppError::unauthorized().at_site(&site));
    }

    let sites = sqlx::query!(
        "SELECT id, domain, site_mask_bit, requires_auth FROM sites"
    )
    .fetch_all(&pool)
    .await?
    .into_iter()
    .map(|row| SiteResponse {
        id: row.id,
        domain: row.domain,
        site_mask_bit: row.site_mask_bit,
        requires_auth: row.requires_auth.unwrap_or(false),
    })
    .collect();

    Ok(Json(sites))
}

#[derive(Deserialize)]
pub struct CreateSiteRequest {
    pub domain: String,
    pub requires_auth: bool,
}

pub async fn create_site(
    State(pool): State<PgPool>,
    site: SiteIdentity,
    Json(payload): Json<CreateSiteRequest>,
) -> Result<Json<SiteResponse>, AppError> {
    if !site.domain.starts_with("localhost") && !site.domain.starts_with("127.0.0.1") {
        return Err(AppError::unauthorized().at_site(&site));
    }

    let existing_sites = sqlx::query!("SELECT site_mask_bit FROM sites")
        .fetch_all(&pool)
        .await?;

    let mut next_bit = existing_sites
        .iter()
        .map(|s| s.site_mask_bit)
        .max()
        .unwrap_or(0);

    if next_bit == 0 {
        next_bit = 1;
    } else {
        next_bit <<= 1;
    }

    let row = sqlx::query!(
        r#"
        INSERT INTO sites (domain, site_mask_bit, requires_auth)
        VALUES ($1, $2, $3)
        RETURNING id, domain, site_mask_bit, requires_auth
        "#,
        payload.domain,
        next_bit,
        payload.requires_auth
    )
    .fetch_one(&pool)
    .await?;

    Ok(Json(SiteResponse {
        id: row.id,
        domain: row.domain,
        site_mask_bit: row.site_mask_bit,
        requires_auth: row.requires_auth.unwrap_or(false),
    }))
}

pub async fn delete_site(
    State(pool): State<PgPool>,
    site: SiteIdentity,
    Path(id): Path<i32>,
) -> Result<StatusCode, AppError> {
    if !site.domain.starts_with("localhost") && !site.domain.starts_with("127.0.0.1") {
        return Err(AppError::unauthorized().at_site(&site));
    }

    sqlx::query!("DELETE FROM sites WHERE id = $1", id)
        .execute(&pool)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}
