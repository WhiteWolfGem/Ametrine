use crate::{error::AppError, extractors::SiteIdentity, models::Tag};
use crate::params::SearchParams;
use axum::{
    Json,
    extract::{Path, Query, State, rejection::QueryRejection},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

#[derive(Deserialize, Debug, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum TagSort {
    Usage,
    Popularity,
    Alphabetical,
}

#[derive(Serialize)]
pub struct TagResponse {
    pub name: String,
    pub uuid: uuid::Uuid,
}

pub async fn admin_fetch_tags(
    State(pool): State<PgPool>,
    site: SiteIdentity,
    params: Result<Query<SearchParams<TagSort>>, QueryRejection>,
) -> Result<Json<Vec<Tag>>, AppError> {
    if !site.requires_auth {
        return Err(AppError::unauthorized().at_site(&site));
    }

    let Query(params) = params.map_err(|e| {
        AppError::bad_request()
            .with_debug(e.to_string())
            .at_site(&site)
    })?;

    let order_col = match params.sort() {
        Some(TagSort::Popularity) => "selected_count",
        Some(TagSort::Usage) => "use_count",
        _ => "tag_name",
    };

    let direction = params.sort_by().to_sql();
    let search_pattern = params.search().map(|s| format!("%{}%", s));

    let query = format!(
        "SELECT tag_name, tag_uuid, use_count, selected_count, visibility_mask FROM tag_stats
         WHERE ($3::TEXT IS NULL OR tag_name ILIKE $3)
         ORDER BY {} {}, tag_name ASC
         LIMIT $1 OFFSET $2",
        order_col, direction
    );

    let tags = sqlx::query_as::<_, Tag>(&query)
        .bind(params.limit())
        .bind(params.offset())
        .bind(search_pattern)
        .fetch_all(&pool)
        .await
        .map_err(|e| AppError::from(e).at_site(&site))?;

    Ok(Json(tags))
}

pub async fn fetch_tags(
    State(pool): State<PgPool>,
    site: SiteIdentity,
    params: Result<Query<SearchParams<TagSort>>, QueryRejection>,
) -> Result<Json<Vec<TagResponse>>, AppError> {
    let Query(params) = params.map_err(|e| {
        AppError::bad_request()
            .with_debug(e.to_string())
            .at_site(&site)
    })?;

    let order_col = match params.sort() {
        Some(TagSort::Popularity) => "selected_count",
        Some(TagSort::Usage) => "use_count",
        _ => "tag_name",
    };

    let direction = params.sort_by().to_sql();
    let search_pattern = params.search().map(|s| format!("%{}%", s));

    let query = format!(
        "SELECT tag_name, tag_uuid FROM tag_stats
         WHERE (visibility_mask & $1) > 0
         AND ($4::TEXT IS NULL OR tag_name ILIKE $4)
         ORDER BY {} {}, tag_name ASC
         LIMIT $2 OFFSET $3",
        order_col, direction
    );

    let tags = sqlx::query_as::<_, (String, uuid::Uuid)>(&query)
        .bind(site.mask)
        .bind(params.limit())
        .bind(params.offset())
        .bind(search_pattern)
        .fetch_all(&pool)
        .await
        .map_err(|e| AppError::from(e).at_site(&site))?
        .into_iter()
        .map(|(name, uuid)| TagResponse { name, uuid })
        .collect();

    Ok(Json(tags))
}

pub async fn increment_tag_selection(
    State(pool): State<PgPool>,
    site: SiteIdentity,
    Path(identifier): Path<String>,
) -> Result<StatusCode, AppError> {
    let tag_uuid = uuid::Uuid::parse_str(&identifier).map_err(|e| {
        AppError::bad_request()
            .with_debug(e.to_string())
            .at_site(&site)
    })?;

    let result = sqlx::query!(
        r#"
            UPDATE tag_stats
            SET selected_count = selected_count + 1
            WHERE tag_uuid = $1
            AND (visibility_mask & $2) > 0
        "#,
        tag_uuid,
        site.mask
    )
    .execute(&pool)
    .await
    .map_err(|e| AppError::from(e).at_site(&site))?;

    if result.rows_affected() == 0 {
        return Err(AppError::not_found().at_site(&site));
    }
    Ok(StatusCode::NO_CONTENT)
}
