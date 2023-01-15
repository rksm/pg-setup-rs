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

mod db;
mod db_cmd_strategy;
mod db_url;
mod error;
mod shell;

#[cfg(feature = "sqlx")]
mod db_sqlx_strategy;
mod table_builder;

pub use db::{PostgresDB, PostgresDBBuilder};
pub use error::{Error, Result};

#[macro_use]
extern crate tracing;

#[macro_use]
extern crate async_trait;
