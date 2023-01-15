use crate::error;

pub(crate) struct SqlxStrategy;

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
