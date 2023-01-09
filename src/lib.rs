//! Very simple helper to create and drop Postgres databases. Useful for tests.
//!
//! Note: This expects that you have the [sqlx-cli](https://github.com/launchbadge/sqlx)
//! installed: `cargo install sqlx-cli`.
//!
//! Example:
//! ```rust
//! # use pg_test_utilities::PostgresDB;
//! # async fn xxx() -> Result<(), Box<dyn std::error::Error>> {
//! static TEST_DB: &str = "postgresql://postgres:@localhost:5432/my_test_db";
//! let mut db = PostgresDBBuilder::new(TEST_DB).start()?;
//! let mut con = db.con().await?;
//!
//! /* test stuff here using con */
//!
//! // when the db struct drops it will automatically drop my_test_db
//! # Ok(())
//! # }
//! ```
//!
//! In case you want to keep the db around for debugging you can call [`PostgresDB::keep_db`].
//!
//! Will use the `public` schema by default but you can set this with [`PostgresDB::schema`].

mod db_url;
mod error;
mod shell;

#[macro_use]
extern crate tracing;

use sqlx::postgres::PgPoolOptions;
use sqlx::{postgres::PgConnectOptions, prelude::*};
use std::str::FromStr;

/// Builder to construct a [`PostgresDB`].
pub struct PostgresDBBuilder {
    db_uri: String,
    keep_db: bool,
    use_sqlx: bool,
    schema: String,
}

impl PostgresDBBuilder {
    pub fn new(db_uri: impl ToString) -> Self {
        Self {
            db_uri: db_uri.to_string(),
            keep_db: false,
            use_sqlx: false,
            schema: "public".to_string(),
        }
    }

    /// When set, does not drop the DB when [`PostgresDB`] goes out of scope.
    #[must_use]
    pub fn keep_db(mut self) -> Self {
        self.keep_db = true;
        self
    }

    /// The postgres schema to use. Defaults to `"public"`.
    #[must_use]
    pub fn schema(mut self, schema: impl ToString) -> Self {
        self.schema = schema.to_string();
        self
    }

    /// Use sqlx to create / drop the database? Requires that sqlx-cli is installed
    /// and in `$PATH` and that migrations are defined. Defaults to false.
    #[must_use]
    pub fn use_sqlx(mut self) -> Self {
        self.use_sqlx = true;
        self
    }

    pub async fn start(self) -> error::Result<PostgresDB> {
        let db = PostgresDB {
            db_uri: self.db_uri,
            keep_db: self.keep_db,
            schema: self.schema,
            strategy: if self.use_sqlx {
                Box::new(SqlxStrategy)
            } else {
                Box::new(CmdStrategy)
            },
        };
        db.setup().await?;
        Ok(db)
    }
}

/// See module comment.
pub struct PostgresDB {
    db_uri: String,
    keep_db: bool,
    schema: String,
    strategy: Box<dyn DBStrategy>,
}

impl PostgresDB {
    pub async fn con(&mut self) -> error::Result<sqlx::PgConnection> {
        Ok(PgConnectOptions::from_str(&self.db_uri)
            .unwrap()
            .disable_statement_logging()
            .connect()
            .await?)
    }

    pub async fn pool(&self) -> error::Result<sqlx::Pool<sqlx::Postgres>> {
        Ok(PgPoolOptions::new()
            .max_connections(5)
            .connect(&self.db_uri)
            .await?)
    }

    pub async fn delete_all_tables(&self) -> error::Result<()> {
        info!("WILL DELETE ALL TABLES");

        let pool = self.pool().await?;
        let mut t = pool.begin().await?;

        sqlx::query("SET TRANSACTION ISOLATION LEVEL SERIALIZABLE")
            .execute(&mut t)
            .await?;

        let tables: Vec<(String,)> = sqlx::query_as(
            "
SELECT table_name
FROM information_schema.tables
WHERE table_schema LIKE $1 AND table_type = 'BASE TABLE';",
        )
        .bind(&self.schema)
        .fetch_all(&mut t)
        .await?;

        for (table,) in tables {
            sqlx::query(&(format!("delete from {table};")))
                .execute(&mut t)
                .await?;
        }
        t.commit().await?;

        Ok(())
    }

    pub async fn setup(&self) -> error::Result<()> {
        self.strategy.setup(&self.db_uri)
    }

    fn tear_down(&self) -> error::Result<()> {
        if self.keep_db {
            return Ok(());
        }

        self.strategy.tear_down(&self.db_uri)
    }
}

impl Drop for PostgresDB {
    fn drop(&mut self) {
        self.tear_down().expect("teardown");
    }
}

// -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-

trait DBStrategy {
    fn setup(&self, db_uri: &str) -> error::Result<()>;
    fn tear_down(&self, db_uri: &str) -> error::Result<()>;
}

struct SqlxStrategy;

impl DBStrategy for SqlxStrategy {
    fn setup(&self, db_uri: &str) -> error::Result<()> {
        info!("Creating {db_uri}");
        shell::cmd(format!("sqlx database setup --database-url {db_uri}",))?.wait()?;
        Ok(())
    }

    fn tear_down(&self, db_uri: &str) -> error::Result<()> {
        info!("Dropping {db_uri}");
        shell::cmd(format!("sqlx database drop --database-url {db_uri} -y"))?;
        Ok(())
    }
}

// -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-

struct CmdStrategy;

impl DBStrategy for CmdStrategy {
    fn setup(&self, db_uri: &str) -> error::Result<()> {
        info!("Creating {db_uri}");

        let db_url: db_url::PgDbUrl = db_uri.parse()?;
        let db = db_url.database;
        let maintenance_url = db_url::maintenance_url(db_uri);
        shell::cmd_with_args(
            "psql",
            [
                maintenance_url,
                "-c".to_string(),
                format!("CREATE DATABASE {db}"),
            ],
        )?
        .wait()?;

        Ok(())
    }

    fn tear_down(&self, db_uri: &str) -> error::Result<()> {
        info!("Dropping {db_uri}");

        let db_url: db_url::PgDbUrl = db_uri.parse()?;
        let db = db_url.database;
        let maintenance_url = db_url::maintenance_url(db_uri);

        // make sure all connections are closed
        shell::cmd_with_args(
            "psql",
            [
                maintenance_url.clone(),
                "-c".to_string(),
                format!(
                    r#"
SELECT pg_terminate_backend(pg_stat_activity.pid)
FROM pg_stat_activity
WHERE pg_stat_activity.datname = '{db}'
  AND pid <> pg_backend_pid();
"#
                ),
            ],
        )?
        .wait()?;

        shell::cmd_with_args(
            "psql",
            [
                maintenance_url,
                "-c".to_string(),
                format!("DROP DATABASE {db}"),
            ],
        )?
        .wait()?;
        Ok(())
    }
}
