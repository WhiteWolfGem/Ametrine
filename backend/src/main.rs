mod config;
mod db;
mod error;
mod extractors;
mod gpg;
mod models;
mod params;
mod routes;
use crate::config::AppConfig;
use axum::extract::FromRef;

#[derive(Clone)]
pub struct AppState {
    pub db: sqlx::PgPool,
    pub config: AppConfig,
}

impl FromRef<AppState> for sqlx::PgPool {
    fn from_ref(state: &AppState) -> Self {
        state.db.clone()
    }
}

impl FromRef<AppState> for AppConfig {
    fn from_ref(state: &AppState) -> Self {
        state.config.clone()
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let settings = AppConfig::load().expect("Failed to load config.toml");

    let pool = db::setup_database(&settings).await?;
    let state = AppState {
        db: pool,
        config: settings.clone(),
    };
    let app = routes::create_router(state);

    let listener = tokio::net::TcpListener::bind(&settings.server_addr)
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
