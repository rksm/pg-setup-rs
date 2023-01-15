use pg_admin::{PostgresDBBuilder, Result};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::builder().parse_lossy("info,pg_admin=debug"),
        )
        .init();

    let db_uri = "postgres://localhost:5432/pg_admin_example";

    let db = PostgresDBBuilder::new(db_uri)
        .schema("public")
        // .keep_db()
        .start()
        .await?;

    db.create_table("users", |t| {
        t.add_column("id", "uuid", |c| c.primary_key());
        t.add_column("name", "text", |c| c.not_null());
        t.add_column("email", "text", |c| c.not_null());
        t.add_column("created_at", "timestamp", |c| c.not_null());
    })
    .await?;

    db.execute("SELECT table_schema,table_name, table_type FROM information_schema.tables WHERE table_schema = 'public';").await?;

    Ok(())
}
