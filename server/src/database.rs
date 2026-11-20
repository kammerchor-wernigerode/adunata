use std::fmt;
use std::io;
use std::time::Duration;

use sqlx::PgPool;
use sqlx::migrate::{MigrateError, Migrator};
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};

static MIGRATOR: Migrator = sqlx::migrate!("./migrations");
const ACQUIRE_TIMEOUT: Duration = Duration::from_secs(5);

/// `EX_DATAERR` from `sysexits.h`: the data in the database does not fit this build.
const DATA_ERROR: i32 = 65;
/// `EX_TEMPFAIL` from `sysexits.h`: the connection failed for a reason that a later attempt may not meet.
const TEMPORARY_FAILURE: i32 = 75;
const FAILURE: i32 = 1;

#[derive(Debug)]
pub enum StartupError {
    Unreachable(sqlx::Error),
    SchemaTooNew { database: i64, supported: i64 },
    Migration(MigrateError),
}

impl StartupError {
    /// The exit status that tells a supervisor whether a restart can help.
    pub fn exit_status(&self) -> i32 {
        match self {
            Self::Unreachable(error) => exit_status(error),
            Self::SchemaTooNew { .. } => DATA_ERROR,
            Self::Migration(error) => migration_exit_status(error),
        }
    }
}

/// The exit status for a database error outside a migration: 75 if it is temporary, 1 otherwise.
pub fn exit_status(error: &sqlx::Error) -> i32 {
    if is_temporary(&Cause::of(error)) {
        TEMPORARY_FAILURE
    } else {
        FAILURE
    }
}

fn migration_exit_status(error: &MigrateError) -> i32 {
    match error {
        MigrateError::VersionMissing(_)
        | MigrateError::VersionMismatch(_)
        | MigrateError::VersionNotPresent(_)
        | MigrateError::VersionTooOld(..)
        | MigrateError::VersionTooNew(..)
        | MigrateError::Dirty(_) => DATA_ERROR,
        MigrateError::Execute(error) | MigrateError::ExecuteMigration(error, _) => {
            statement_exit_status(&Cause::of(error))
        }
        _ => FAILURE,
    }
}

fn statement_exit_status(cause: &Cause) -> i32 {
    if is_temporary(cause) {
        TEMPORARY_FAILURE
    } else if violates_the_data(cause) {
        DATA_ERROR
    } else {
        FAILURE
    }
}

/// The parts of a `sqlx::Error` that decide its exit status.
#[derive(Debug, PartialEq)]
enum Cause {
    Io(io::ErrorKind),
    Pool,
    Database(Option<String>),
    Other,
}

impl Cause {
    fn of(error: &sqlx::Error) -> Self {
        match error {
            sqlx::Error::Io(error) => Self::Io(error.kind()),
            sqlx::Error::PoolTimedOut | sqlx::Error::PoolClosed | sqlx::Error::WorkerCrashed => {
                Self::Pool
            }
            sqlx::Error::Database(error) => {
                Self::Database(error.code().map(std::borrow::Cow::into_owned))
            }
            _ => Self::Other,
        }
    }
}

/// Whether a later attempt may succeed. `InvalidData` is the exception among I/O errors, because the connection was made and the peer was rejected.
fn is_temporary(cause: &Cause) -> bool {
    match cause {
        Cause::Io(kind) => *kind != io::ErrorKind::InvalidData,
        Cause::Pool => true,
        Cause::Database(code) => has_class(code.as_deref(), &["08", "53", "57"]),
        Cause::Other => false,
    }
}

/// Whether a statement failed on the data or the schema in the database. A missing privilege (`42501`) is excluded, because the operator fixes the role, not the data.
fn violates_the_data(cause: &Cause) -> bool {
    match cause {
        Cause::Database(code) => {
            has_class(code.as_deref(), &["22", "23", "42"]) && code.as_deref() != Some("42501")
        }
        _ => false,
    }
}

fn has_class(code: Option<&str>, classes: &[&str]) -> bool {
    code.and_then(|code| code.get(..2))
        .is_some_and(|class| classes.contains(&class))
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
mod tests {
    use super::*;

    fn database(code: &str) -> Cause {
        Cause::Database(Some(code.to_owned()))
    }

    #[test]
    fn io_errors_are_temporary_except_invalid_data() {
        for kind in [
            io::ErrorKind::ConnectionRefused,
            io::ErrorKind::NotFound,
            io::ErrorKind::TimedOut,
            io::ErrorKind::Other,
        ] {
            let error = sqlx::Error::Io(io::Error::from(kind));
            assert_eq!(exit_status(&error), 75, "{kind:?}");
        }
        let invalid = sqlx::Error::Io(io::Error::from(io::ErrorKind::InvalidData));
        assert_eq!(exit_status(&invalid), 1);
    }

    #[test]
    fn pool_errors_are_temporary() {
        for error in [
            sqlx::Error::PoolTimedOut,
            sqlx::Error::PoolClosed,
            sqlx::Error::WorkerCrashed,
        ] {
            assert_eq!(exit_status(&error), 75, "{error}");
        }
    }

    #[test]
    fn other_errors_fail() {
        let tls = sqlx::Error::Tls("server does not support TLS".into());
        assert_eq!(exit_status(&tls), 1);
        assert_eq!(exit_status(&sqlx::Error::RowNotFound), 1);
    }

    #[test]
    fn sqlstates_of_connection_resources_and_shutdown_are_temporary() {
        for code in ["08006", "53300", "57P01", "57P03"] {
            assert!(is_temporary(&database(code)), "{code}");
        }
        for code in ["28P01", "3D000", "42501", "23505"] {
            assert!(!is_temporary(&database(code)), "{code}");
        }
        assert!(!is_temporary(&Cause::Database(None)));
    }

    #[test]
    fn migration_statuses_by_sqlstate() {
        assert_eq!(statement_exit_status(&database("22P02")), 65);
        assert_eq!(statement_exit_status(&database("23505")), 65);
        assert_eq!(statement_exit_status(&database("42P07")), 65);
        assert_eq!(statement_exit_status(&database("42501")), 1);
        assert_eq!(statement_exit_status(&database("57P01")), 75);
        assert_eq!(statement_exit_status(&database("28P01")), 1);
        assert_eq!(
            statement_exit_status(&Cause::Io(io::ErrorKind::ConnectionReset)),
            75
        );
        assert_eq!(
            statement_exit_status(&Cause::Io(io::ErrorKind::InvalidData)),
            1
        );
    }

    #[test]
    fn migration_errors_without_a_statement() {
        for error in [
            MigrateError::VersionMissing(1),
            MigrateError::VersionMismatch(1),
            MigrateError::VersionNotPresent(1),
            MigrateError::VersionTooOld(1, 2),
            MigrateError::VersionTooNew(2, 1),
            MigrateError::Dirty(1),
        ] {
            assert_eq!(StartupError::Migration(error).exit_status(), 65);
        }
        assert_eq!(
            StartupError::Migration(MigrateError::Execute(sqlx::Error::PoolTimedOut)).exit_status(),
            75
        );
        assert_eq!(
            StartupError::Migration(MigrateError::Source("unreadable".into())).exit_status(),
            1
        );
    }

    #[test]
    fn startup_errors() {
        let too_new = StartupError::SchemaTooNew {
            database: 2,
            supported: 1,
        };
        assert_eq!(too_new.exit_status(), 65);
        assert_eq!(
            StartupError::Unreachable(sqlx::Error::PoolTimedOut).exit_status(),
            75
        );
        assert_eq!(
            StartupError::Unreachable(sqlx::Error::Tls("rejected".into())).exit_status(),
            1
        );
    }
}

#[cfg(test)]
pub mod testing {
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::thread;

    use sqlx::postgres::PgConnectOptions;
    use sqlx::{AssertSqlSafe, Connection, PgConnection, PgPool};

    use crate::configuration::test_setting;

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

    /// Creates an empty database without any schema on the server of `ADNT_DATABASE_URL`.
    pub async fn empty_database() -> TestDatabase {
        let server: PgConnectOptions = test_setting("ADNT_DATABASE_URL")
            .expect("ADNT_DATABASE_URL must name the test database server")
            .parse()
            .expect("ADNT_DATABASE_URL must be a PostgreSQL URL");
        let name = format!(
            "adunata_test_{}_{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        );
        let mut admin = PgConnection::connect_with(&server)
            .await
            .expect("test database server must be reachable through ADNT_DATABASE_URL");
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
mod integration {
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
        assert!(table_exists(&pool, "responses").await);
        assert!(table_exists(&pool, "mail_outbox").await);
    }

    #[tokio::test]
    async fn start_after_an_update() {
        let database = empty_database().await;
        let options = database.options();
        let first = MIGRATOR
            .iter()
            .map(|migration| migration.version)
            .min()
            .unwrap();
        let older = PgPoolOptions::new()
            .connect_with(options.clone())
            .await
            .unwrap();
        MIGRATOR.run_to(first, &older).await.unwrap();
        assert!(!table_exists(&older, "responses").await);
        older.close().await;

        let pool = connect(options).await.unwrap();
        assert!(table_exists(&pool, "responses").await);
        assert_eq!(
            applied_version(&pool).await.unwrap(),
            Some(latest_supported_version())
        );
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
        assert_eq!(error.exit_status(), 65);
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

    #[tokio::test]
    async fn start_after_an_applied_migration_changed() {
        let database = fresh_database().await;
        let pool = database.pool();
        sqlx::query("UPDATE _sqlx_migrations SET checksum = '\\x00' WHERE version = (SELECT min(version) FROM _sqlx_migrations)")
            .execute(&pool)
            .await
            .unwrap();
        let options = pool.connect_options().as_ref().clone();

        let error = connect(options).await.expect_err("start must be refused");
        assert!(
            matches!(
                error,
                StartupError::Migration(MigrateError::VersionMismatch(_))
            ),
            "{error}"
        );
        assert_eq!(error.exit_status(), 65);
    }

    #[tokio::test]
    async fn start_against_a_database_that_does_not_exist() {
        let database = empty_database().await;
        let options = database.options().database("adunata_test_missing");

        let error = connect(options).await.expect_err("start must be refused");
        assert_eq!(error.exit_status(), 1, "{error}");
    }
}
