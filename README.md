# pg-admin

Simple helper to create and drop Postgres databases. Useful for tests and for programmatically setting up DBs.

Note: For now this makes use of sqlx, in particular it expects that you have
[sqlx-cli](https://github.com/launchbadge/sqlx) installed: `cargo install sqlx-cli`.

Example:
```rust
static TEST_DB: &str = "postgresql://postgres:@localhost:5432/my_test_db";
let mut db = PostgresDB::start(TEST_DB)?;
let mut con = db.con().await?;

/* test stuff here using con */

// when the db struct drops it will automatically drop my_test_db
```

In case you want to keep the db around for debugging you can call [`PostgresDB::keep_db`].

Will use the `public` schema by default but you can set this with [`PostgresDB::schema`].
