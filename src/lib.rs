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
//! let mut db = PostgresDB::start(TEST_DB)?;
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

use sqlx::{postgres::PgConnectOptions, prelude::*};
use std::str::FromStr;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub fn cmd(cmd: impl AsRef<str>) -> Result<()> {
    let mut args = cmd.as_ref().split(' ');
    let cmd = args.next().unwrap();
    let result = std::process::Command::new(cmd)
        .args(args)
        .spawn()
        .and_then(|proc| proc.wait_with_output())?;
    let stdout = String::from_utf8_lossy(&result.stdout);
    let stderr = String::from_utf8_lossy(&result.stderr);
    print!("{}", stdout);
    print!("{}", stderr);
    Ok(())
}

/// See module comment.
pub struct PostgresDB {
    db_uri: String,
    keep_db: bool,
    schema: String,
}

impl PostgresDB {
    pub fn start(db_uri: impl ToString) -> Result<Self> {
        let db_uri = db_uri.to_string();
        tracing::info!("Creating {db_uri}");
        cmd(format!("sqlx database setup --database-url {db_uri}"))?;
        Ok(Self {
            db_uri,
            keep_db: false,
            schema: "public".to_string(),
        })
    }

    pub fn schema(&mut self, schema: impl ToString) {
        self.schema = schema.to_string();
    }

    pub fn keep_db(&mut self) {
        self.keep_db = true;
    }

    pub async fn con(&mut self) -> Result<sqlx::PgConnection> {
        Ok(PgConnectOptions::from_str(&self.db_uri)
            .unwrap()
            .disable_statement_logging()
            .connect()
            .await?)
    }

    pub async fn pool(&self) -> Result<sqlx::Pool<sqlx::Postgres>> {
        Ok(sqlx::postgres::PgPoolOptions::new()
            .max_connections(5)
            .connect(&self.db_uri)
            .await?)
    }

    pub async fn delete_all_tables(&self) -> Result<()> {
        eprintln!("WILL DELETE ALL TABLES");
        let pool = self.pool().await?;
        let mut t = pool.begin().await?;
        sqlx::query("SET TRANSACTION ISOLATION LEVEL SERIALIZABLE")
            .execute(&mut t)
            .await?;
        let tables: Vec<(String,)> = sqlx::query_as(
            "
SELECT table_name
FROM information_schema.tables
WHERE table_name LIKE 'twitter_%' AND table_type = 'BASE TABLE';",
        )
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
}

impl Drop for PostgresDB {
    fn drop(&mut self) {
        if !self.keep_db {
            let db_uri = &self.db_uri;
            tracing::info!("Dropping {db_uri}");
            cmd(format!("sqlx database drop --database-url {db_uri} -y")).unwrap();
        }
    }
}
