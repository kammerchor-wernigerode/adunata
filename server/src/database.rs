use sqlx::PgPool;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};

pub async fn connect(options: PgConnectOptions) -> Result<PgPool, sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    Ok(pool)
}

#[cfg(test)]
pub mod testing {
    use std::sync::atomic::{AtomicU32, Ordering};

    use sqlx::postgres::PgConnectOptions;
    use sqlx::{AssertSqlSafe, Connection, PgConnection, PgPool};

    static NEXT: AtomicU32 = AtomicU32::new(0);

    /// Creates an empty database with all migrations applied. The server comes from the libpq environment variables.
    pub async fn fresh_database() -> PgPool {
        let server = PgConnectOptions::new();
        let name = format!(
            "adunata_test_{}_{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        );
        let mut admin = PgConnection::connect_with(&server)
            .await
            .expect("test database server must be reachable through PGHOST, PGPORT and PGUSER");
        sqlx::raw_sql(AssertSqlSafe(format!(r#"CREATE DATABASE "{name}""#)))
            .execute(&mut admin)
            .await
            .expect("test user must be allowed to create databases");
        super::connect(server.database(&name))
            .await
            .expect("migrations must apply to an empty database")
    }
}

#[cfg(test)]
mod tests {
    use super::testing::fresh_database;

    #[tokio::test]
    async fn migrations_apply_to_an_empty_database() {
        let pool = fresh_database().await;
        let tables: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM information_schema.tables WHERE table_name = 'spike_responses'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(tables, 1);
    }
}
