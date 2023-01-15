use crate::{db_cmd_strategy::CmdStrategy, error::Result, table_builder::TableBuilder};

/// Trait is used to implement actions of [`PostgresDB`].
#[async_trait]
pub(crate) trait DBStrategy {
    fn setup(&self, db_uri: &str) -> Result<()>;
    fn tear_down(&self, db_uri: &str) -> Result<()>;
    async fn execute(&self, db_uri: &str, sql: &str) -> Result<()>;
}

/// Builder to construct a [`PostgresDB`].
pub struct PostgresDBBuilder {
    db_uri: String,
    keep_db: bool,
    #[cfg(feature = "sqlx")]
    use_sqlx: bool,
    schema: String,
}

impl PostgresDBBuilder {
    pub fn new(db_uri: impl ToString) -> Self {
        Self {
            db_uri: db_uri.to_string(),
            keep_db: false,
            #[cfg(feature = "sqlx")]
            use_sqlx: false,
            schema: "public".to_string(),
        }
    }

    /// When set, does not drop the DB when [`PostgresDB`] goes out of scope.
    #[must_use]
    pub fn keep_db(mut self) -> Self {
        self.keep_db = true;
        self
    }

    /// The postgres schema to use. Defaults to `"public"`.
    #[must_use]
    pub fn schema(mut self, schema: impl ToString) -> Self {
        self.schema = schema.to_string();
        self
    }

    /// Use sqlx to create / drop the database? Requires that sqlx-cli is installed
    /// and in `$PATH` and that migrations are defined. Defaults to false.
    #[must_use]
    #[cfg(feature = "sqlx")]
    pub fn use_sqlx(mut self) -> Self {
        self.use_sqlx = true;
        self
    }

    pub async fn start(self) -> Result<PostgresDB> {
        #[cfg(feature = "sqlx")]
        let strategy: Box<dyn DBStrategy> = if self.use_sqlx {
            Box::new(crate::db_sqlx_strategy::SqlxStrategy)
        } else {
            Box::new(CmdStrategy)
        };
        #[cfg(not(feature = "sqlx"))]
        let strategy = Box::new(CmdStrategy);
        let db = PostgresDB {
            db_uri: self.db_uri,
            keep_db: self.keep_db,
            schema: self.schema,
            strategy,
        };
        db.setup().await?;
        Ok(db)
    }
}

/// See module comment.
pub struct PostgresDB {
    db_uri: String,
    keep_db: bool,
    schema: String,
    strategy: Box<dyn DBStrategy>,
}

impl PostgresDB {
    #[cfg(feature = "sqlx")]
    pub async fn con(&mut self) -> Result<sqlx::PgConnection> {
        crate::db_sqlx_strategy::connect(&self.db_uri).await
    }

    #[cfg(feature = "sqlx")]
    pub async fn pool(&self) -> Result<sqlx::Pool<sqlx::Postgres>> {
        crate::db_sqlx_strategy::pool(&self.db_uri).await
    }

    #[cfg(feature = "sqlx")]
    pub async fn delete_all_tables(&self) -> Result<()> {
        // use sqlx::postgres::PgPoolOptions;
        // use sqlx::{postgres::PgConnectOptions, prelude::*};

        info!("WILL DELETE ALL TABLES");

        let pool = self.pool().await?;
        let mut t = pool.begin().await?;

        sqlx::query("SET TRANSACTION ISOLATION LEVEL SERIALIZABLE")
            .execute(&mut t)
            .await?;

        let tables: Vec<(String,)> = sqlx::query_as(
            "
    SELECT table_name
    FROM information_schema.tables
    WHERE table_schema LIKE $1 AND table_type = 'BASE TABLE';",
        )
        .bind(&self.schema)
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

    pub async fn execute(&self, sql: &str) -> Result<()> {
        self.strategy.execute(&self.db_uri, sql).await
    }

    pub async fn create_table<F>(&self, table_name: impl AsRef<str>, tbl_callback: F) -> Result<()>
    where
        F: Fn(&mut TableBuilder),
    {
        let mut table = TableBuilder::new(format!("{}.{}", self.schema, table_name.as_ref()));
        tbl_callback(&mut table);
        let sql = table.to_sql()?;
        self.execute(sql.as_str()).await
    }

    pub async fn setup(&self) -> Result<()> {
        self.strategy.setup(&self.db_uri)
    }

    fn tear_down(&self) -> Result<()> {
        if self.keep_db {
            return Ok(());
        }

        self.strategy.tear_down(&self.db_uri)
    }
}

impl Drop for PostgresDB {
    fn drop(&mut self) {
        self.tear_down().expect("teardown");
    }
}
