use crate::config::AppConfig;
use sqlx::PgPool;

pub async fn setup_database(config: &AppConfig) -> anyhow::Result<PgPool> {
    let pool = PgPool::connect(&config.database_url).await?;

    if config.run_migrations {
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("Failed to migrate the database");
        println!("Migrations executed");
    }

    sync_sites(&pool, config).await?;

    Ok(pool)
}

pub async fn sync_sites(pool: &PgPool, config: &AppConfig) -> anyhow::Result<()> {
    let existing_sites = sqlx::query!("SELECT domain, site_mask_bit FROM sites")
        .fetch_all(pool)
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

    for site in &config.sites {
        let existing = existing_sites.iter().find(|s| s.domain == site.domain);

        let mask_to_use = match existing {
            Some(s) => s.site_mask_bit,
            None => {
                let bit = next_bit;
                next_bit <<= 1;
                bit
            }
        };

        sqlx::query!(
            r#"
        INSERT INTO sites (domain, site_mask_bit, requires_auth)
        VALUES ($1, $2, $3)
        On conflict (domain) DO UPDATE 
        SET requires_auth = EXCLUDED.requires_auth
        "#,
            site.domain,
            mask_to_use,
            site.auth
        )
        .execute(pool)
        .await?;
    }
    Ok(())
}
