use crate::{db::DBStrategy, error::Result, shell};

pub(crate) struct SqlxStrategy;

pub(crate) async fn connect(db_uri: &str) -> Result<sqlx::PgConnection> {
    use sqlx::ConnectOptions;
    use std::str::FromStr;
    Ok(sqlx::postgres::PgConnectOptions::from_str(db_uri)
        .unwrap()
        .disable_statement_logging()
        .connect()
        .await?)
}

#[cfg(feature = "sqlx")]
pub(crate) async fn pool(db_uri: &str) -> Result<sqlx::Pool<sqlx::Postgres>> {
    Ok(sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(db_uri)
        .await?)
}

#[async_trait]
impl DBStrategy for SqlxStrategy {
    fn setup(&self, db_uri: &str) -> Result<()> {
        info!("Creating {db_uri}");
        shell::cmd(format!("sqlx database setup --database-url {db_uri}",))?.wait()?;
        Ok(())
    }

    fn tear_down(&self, db_uri: &str) -> Result<()> {
        info!("Dropping {db_uri}");
        shell::cmd(format!("sqlx database drop --database-url {db_uri} -y"))?;
        Ok(())
    }

    async fn execute(&self, db_uri: &str, sql: &str) -> crate::Result<()> {
        let pool = pool(db_uri).await?;
        sqlx::query(sql).execute(&pool).await?;
        Ok(())
    }
}
