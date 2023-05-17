/*!

[![Crates.io](https://img.shields.io/crates/v/pg-setup)](https://crates.io/crates/pg-setup)
[![](https://docs.rs/pg-setup/badge.svg)](https://docs.rs/pg-setup)
[![License](https://img.shields.io/crates/l/pg-setup?color=informational&logo=mit)](/LICENSE.md)

Simple helper to create and drop Postgres databases. Useful for tests.

This uses either the psql command line utility (default) or the sqlx and sqlx-cli (which makes use of sqlx migrations).
Use the `sqlx` feature for that.

Example:
```rust
# use pg_setup::{PostgresDBBuilder, Result};
#
# #[tokio::main]
# async fn main() -> Result<()> {
    let db_uri = "postgres://localhost:5432/pg_setup_example";

    let db = PostgresDBBuilder::new(db_uri)
        .schema("public")
        // optionally keep db
        .keep_db()
        .start()
        .await?;

    // optionally create a table
    db.create_table("users", |t| {
        t.add_column("id", "uuid", |c| c.primary_key());
        t.add_column("name", "text", |c| c.not_null());
        t.add_column("email", "text", |c| c.not_null());
        t.add_column("created_at", "timestamp", |c| c.not_null());
    })
    .await?;

    // execute sql
    db.execute("SELECT table_schema,table_name, table_type FROM information_schema.tables WHERE table_schema = 'public';").await?;

    // db will be dropped at the end of the scope, unless `keep_db` is called!

#    Ok(())
# }
```

In case you want to keep the db around for debugging you can call [`PostgresDB::keep_db`].

Will use the `public` schema by default but you can set this with [`PostgresDB::schema`].

*/

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
