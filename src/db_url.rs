use super::error::Error;
use std::str::FromStr;

#[allow(unused)]
pub(crate) struct PgDbUrl {
    pub(crate) database: String,
    pub(crate) user: String,
    pub(crate) password: Option<String>,
    pub(crate) host: String,
    pub(crate) port: u16,
}

impl FromStr for PgDbUrl {
    type Err = Error;

    fn from_str(url_string: &str) -> std::result::Result<Self, Self::Err> {
        let url: url::Url = url_string
            .parse()
            .map_err(|err: url::ParseError| Error::PgUrlParseError(err.to_string()))?;

        let scheme = url.scheme();
        if scheme != "postgres" {
            return Err(Error::PgUrlParseError(format!(
                "expected scheme \"postgres\": {url_string}",
            )));
        }

        let user = url.username();
        let user = if user.is_empty() { "postgres" } else { user };
        let password = url.password();
        let host = if let Some(host) = url.host() {
            host
        } else {
            return Err(Error::PgUrlParseError(format!(
                "host missing: {url_string}",
            )));
        };
        let port = url.port().unwrap_or(5432);
        let database = url.path().trim_start_matches('/');

        Ok(Self {
            database: database.to_string(),
            user: user.to_string(),
            password: password.map(|p| p.to_string()),
            host: host.to_string(),
            port,
        })
    }
}

/// Given a `db_url`, return a "maintainance" url that can be used to create or
/// drop the original database.
pub(crate) fn maintenance_url(db_url: impl AsRef<str>) -> String {
    let db_url = db_url.as_ref();

    let opts =
        sqlx::postgres::PgConnectOptions::from_str(db_url).expect("cannot parse postres db ul");

    let database = opts
        .get_database()
        .expect("postgres:// url string does no specify database");
    // switch us to the maintenance database
    // use `postgres` _unless_ the database is postgres, in which case, use `template1`
    // this matches the behavior of the `createdb` util
    let maintenance_db = if database == "postgres" {
        "template1"
    } else {
        "postgres"
    };

    let idx = db_url
        .rfind(database)
        .expect("cannot find {database:?} in the db url");
    let (prefix, _) = db_url.split_at(idx);
    format!("{prefix}{maintenance_db}")
}

#[cfg(test)]
mod tests {
    use super::maintenance_url;

    #[test]
    fn find_maintainance_db() {
        let url = maintenance_url("postgres://x@server/foo");
        assert_eq!(url, "postgres://x@server/postgres");

        let url = maintenance_url("postgres://x@server/postgres");
        assert_eq!(url, "postgres://x@server/template1");

        let url = maintenance_url("postgres://server:1234/x");
        assert_eq!(url, "postgres://server:1234/postgres");
    }
}
