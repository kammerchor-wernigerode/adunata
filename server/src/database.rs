use std::fmt;
use std::time::Duration;

use sqlx::PgPool;
use sqlx::migrate::Migrator;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};

static MIGRATOR: Migrator = sqlx::migrate!("./migrations");
const ACQUIRE_TIMEOUT: Duration = Duration::from_secs(5);

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
        .acquire_timeout(ACQUIRE_TIMEOUT)
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
    use std::thread;

    use sqlx::postgres::PgConnectOptions;
    use sqlx::{AssertSqlSafe, Connection, PgConnection, PgPool};

    static NEXT: AtomicU32 = AtomicU32::new(0);

    /// A database that exists for one test. Dropping it drops the database on the server.
    pub struct TestDatabase {
        server: PgConnectOptions,
        name: String,
    }

    impl TestDatabase {
        pub fn options(&self) -> PgConnectOptions {
            self.server.clone().database(&self.name)
        }
    }

    impl Drop for TestDatabase {
        fn drop(&mut self) {
            let server = self.server.clone();
            let statement = format!(r#"DROP DATABASE "{}" WITH (FORCE)"#, self.name);
            let outcome = thread::spawn(move || {
                tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()?
                    .block_on(execute_on_server(&server, statement))
                    .map_err(std::io::Error::other)
            })
            .join();
            if !matches!(outcome, Ok(Ok(()))) {
                eprintln!("test database {} was not dropped", self.name);
            }
        }
    }

    /// A test database with all migrations applied.
    pub struct FreshDatabase {
        pool: PgPool,
        _database: TestDatabase,
    }

    impl FreshDatabase {
        pub fn pool(&self) -> PgPool {
            self.pool.clone()
        }
    }

    async fn execute_on_server(
        server: &PgConnectOptions,
        statement: String,
    ) -> Result<(), sqlx::Error> {
        let mut admin = PgConnection::connect_with(server).await?;
        sqlx::raw_sql(AssertSqlSafe(statement))
            .execute(&mut admin)
            .await?;
        Ok(())
    }

    /// Creates an empty database without any schema. The server comes from the libpq environment variables.
    pub async fn empty_database() -> TestDatabase {
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
        TestDatabase { server, name }
    }

    /// Creates an empty database with all migrations applied.
    pub async fn fresh_database() -> FreshDatabase {
        let database = empty_database().await;
        let pool = super::connect(database.options())
            .await
            .expect("migrations must apply to an empty database");
        FreshDatabase {
            pool,
            _database: database,
        }
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
        let database = empty_database().await;
        let pool = connect(database.options()).await.unwrap();
        assert!(table_exists(&pool, "spike_responses").await);
        assert!(table_exists(&pool, "mail_outbox").await);
    }

    #[tokio::test]
    async fn start_after_an_update() {
        let database = empty_database().await;
        let options = database.options();
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
        let database = fresh_database().await;
        let pool = database.pool();
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
