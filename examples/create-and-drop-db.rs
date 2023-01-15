use pg_admin::{PostgresDBBuilder, Result};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::builder().parse_lossy("trace"))
        .init();

    let db_uri = "postgres://postgres:postgres@localhost:5432/pg_admin_example";

    let db = PostgresDBBuilder::new(db_uri)
        .schema("public")
        .start()
        .await?;

    Ok(())
}
