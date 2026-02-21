use crate::config::AppConfig;
use crate::error::AppError;
use axum::{
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
};
use sqlx::PgPool;

pub struct SiteIdentity {
    pub mask: i32,
    pub domain: String,
    pub requires_auth: bool,
    pub gpg_email: Option<String>,
}

impl<S> FromRequestParts<S> for SiteIdentity
where
    PgPool: FromRef<S>,
    AppConfig: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let pool = PgPool::from_ref(state);
        let config = AppConfig::from_ref(state);

        let mut host = parts
            .headers
            .get("host")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("unknown")
            .to_string();

        if config.allow_debug_headers {
            if let Some(debug_host) = parts
                .headers
                .get("x-debug-host")
                .and_then(|h| h.to_str().ok())
            {
                host = debug_host.to_string();
            }
        }

        let site = sqlx::query!(
            "SELECT site_mask_bit, requires_auth FROM sites WHERE domain = $1",
            host
        )
        .fetch_optional(&pool)
        .await?
        .ok_or_else(|| AppError::unauthorized())?;

        Ok(SiteIdentity {
            mask: site.site_mask_bit,
            domain: host,
            requires_auth: site.requires_auth.unwrap_or(false),
            gpg_email: config.gpg_email.clone(),
        })
    }
}
