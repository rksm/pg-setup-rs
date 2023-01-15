use crate::{
    db::DBStrategy,
    db_url::{self, PgDbUrl},
    error::Result,
    shell,
};

pub(crate) struct CmdStrategy;

impl DBStrategy for CmdStrategy {
    fn setup(&self, db_uri: &str) -> Result<()> {
        info!("Creating {db_uri}");

        let db_url: PgDbUrl = db_uri.parse()?;
        let db = db_url.database;
        let maintenance_url = db_url::maintenance_url(db_uri)?;
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

    fn tear_down(&self, db_uri: &str) -> Result<()> {
        info!("Dropping {db_uri}");

        let db_url: PgDbUrl = db_uri.parse()?;
        let db = db_url.database;
        let maintenance_url = db_url::maintenance_url(db_uri)?;

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
