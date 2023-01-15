use pg_helper::{PostgresDBBuilder, Result};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::builder().parse_lossy("info,pg_helper=debug"),
        )
        .init();

    let db_uri = "postgres://localhost:5432/pg_helper_example";

    let db = PostgresDBBuilder::new(db_uri)
        .schema("public")
        // .keep_db()
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

    Ok(())
}
