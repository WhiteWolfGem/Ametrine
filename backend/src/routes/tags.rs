use crate::error::AppError;
use crate::extractors::SiteIdentity;
use crate::params::SearchParams;
use axum::{
    Json,
    extract::{Path, Query, State},
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

pub async fn fetch_tags(
    State(pool): State<PgPool>,
    site: SiteIdentity,
    Query(params): Query<SearchParams<TagSort>>,
) -> Result<Json<Vec<TagResponse>>, AppError> {
    let order_col = match params.sort() {
        Some(&TagSort::Popularity) => "selected_count",
        Some(&TagSort::Usage) => "use_count",
        _ => "tag_name",
    };

    let direction = params.sort_by().to_sql();

    let query = format!(
        "SELECT tag_name, tag_uuid FROM tag_stats
         WHERE (visibility_mask & $1) > 0
         AND ($4::TEXT IS NULL OR tag_name ILIKE $4)
         ORDER BY {} {}
         LIMIT $2 OFFSET $3",
        order_col, direction
    );

    let tags = sqlx::query_as::<_, (String, uuid::Uuid)>(&query)
        .bind(site.mask)
        .bind(params.limit())
        .bind(params.offset())
        .fetch_all(&pool)
        .await?
        .into_iter()
        .map(|(name, uuid)| TagResponse { name, uuid })
        .collect();

    Ok(Json(tags))
}

pub async fn increment_tag_selection(
    State(pool): State<PgPool>,
    site: SiteIdentity,
    Path(tag_uuid): Path<uuid::Uuid>,
) -> Result<axum::http::StatusCode, AppError> {
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
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    Ok(axum::http::StatusCode::NO_CONTENT)
}
