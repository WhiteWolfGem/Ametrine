use crate::config::AppConfig;
use sqlx::PgPool;

// Setup the database and execute any migrations
pub async fn setup_database(config: &AppConfig) -> anyhow::Result<PgPool> {
    let pool = PgPool::connect(&config.database_url).await?;

    if config.run_migrations {
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("Failed to migrate the database");
        println!("Migrations executed");
    }

    Ok(pool)
}
