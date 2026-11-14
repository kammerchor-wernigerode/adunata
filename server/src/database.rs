use std::fmt;

use sqlx::PgPool;
use sqlx::migrate::Migrator;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};

static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

#[derive(Debug)]
pub enum StartupError {
    Unreachable(sqlx::Error),
    SchemaTooNew { database: i64, supported: i64 },
    Migration(sqlx::migrate::MigrateError),
}

impl fmt::Display for StartupError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unreachable(error) => write!(formatter, "the database is unreachable: {error}"),
            Self::SchemaTooNew {
                database,
                supported,
            } => write!(
                formatter,
                "the database schema has version {database}, but this release supports versions up to {supported}. Start a release that supports version {database} or restore a backup."
            ),
            Self::Migration(error) => write!(formatter, "a migration failed: {error}"),
        }
    }
}

/// Connects to the database and brings its schema to the latest version this release knows.
pub async fn connect(options: PgConnectOptions) -> Result<PgPool, StartupError> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await
        .map_err(StartupError::Unreachable)?;
    let supported = latest_supported_version();
    if let Some(database) = applied_version(&pool)
        .await
        .map_err(StartupError::Unreachable)?
        && database > supported
    {
        return Err(StartupError::SchemaTooNew {
            database,
            supported,
        });
    }
    MIGRATOR.run(&pool).await.map_err(StartupError::Migration)?;
    Ok(pool)
}

fn latest_supported_version() -> i64 {
    MIGRATOR
        .iter()
        .map(|migration| migration.version)
        .max()
        .unwrap_or(0)
}

async fn applied_version(pool: &PgPool) -> Result<Option<i64>, sqlx::Error> {
    let tracked: bool = sqlx::query_scalar("SELECT to_regclass('_sqlx_migrations') IS NOT NULL")
        .fetch_one(pool)
        .await?;
    if !tracked {
        return Ok(None);
    }
    sqlx::query_scalar("SELECT max(version) FROM _sqlx_migrations WHERE success")
        .fetch_one(pool)
        .await
}

#[cfg(test)]
pub mod testing {
    use std::sync::atomic::{AtomicU32, Ordering};

    use sqlx::postgres::PgConnectOptions;
    use sqlx::{AssertSqlSafe, Connection, PgConnection, PgPool};

    static NEXT: AtomicU32 = AtomicU32::new(0);

    /// Creates an empty database without any schema. The server comes from the libpq environment variables.
    pub async fn empty_database() -> PgConnectOptions {
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
        server.database(&name)
    }

    /// Creates an empty database with all migrations applied.
    pub async fn fresh_database() -> PgPool {
        super::connect(empty_database().await)
            .await
            .expect("migrations must apply to an empty database")
    }
}

#[cfg(test)]
mod tests {
    use super::testing::{empty_database, fresh_database};
    use super::*;

    async fn table_exists(pool: &PgPool, name: &str) -> bool {
        sqlx::query_scalar("SELECT to_regclass($1) IS NOT NULL")
            .bind(name)
            .fetch_one(pool)
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn start_against_an_empty_database() {
        let pool = connect(empty_database().await).await.unwrap();
        assert!(table_exists(&pool, "spike_responses").await);
        assert!(table_exists(&pool, "mail_outbox").await);
    }

    #[tokio::test]
    async fn start_after_an_update() {
        let options = empty_database().await;
        let pool = connect(options.clone()).await.unwrap();
        let latest = latest_supported_version();
        sqlx::query("DROP TABLE mail_outbox")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("DELETE FROM _sqlx_migrations WHERE version = $1")
            .bind(latest)
            .execute(&pool)
            .await
            .unwrap();
        pool.close().await;

        let pool = connect(options).await.unwrap();
        assert!(table_exists(&pool, "mail_outbox").await);
        assert_eq!(applied_version(&pool).await.unwrap(), Some(latest));
    }

    #[tokio::test]
    async fn start_after_a_downgrade() {
        let pool = fresh_database().await;
        let newer = latest_supported_version() + 1;
        sqlx::query(
            "INSERT INTO _sqlx_migrations (version, description, success, checksum, execution_time) VALUES ($1, 'from a newer release', true, '\\x00', 0)",
        )
        .bind(newer)
        .execute(&pool)
        .await
        .unwrap();
        let options = pool.connect_options().as_ref().clone();
        let applied_before: i64 = sqlx::query_scalar("SELECT count(*) FROM _sqlx_migrations")
            .fetch_one(&pool)
            .await
            .unwrap();

        let error = connect(options).await.expect_err("start must be refused");
        let message = error.to_string();
        assert!(message.contains(&newer.to_string()), "{message}");
        assert!(
            message.contains(&latest_supported_version().to_string()),
            "{message}"
        );

        let applied_after: i64 = sqlx::query_scalar("SELECT count(*) FROM _sqlx_migrations")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(applied_after, applied_before);
    }
}
