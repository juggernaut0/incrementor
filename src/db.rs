use postgres::transaction::Transaction;
use r2d2::Pool;
use r2d2_postgres::{PostgresConnectionManager, TlsMode};
use uuid::Uuid;

pub struct DataAccess {
    pool: Pool<PostgresConnectionManager>,
}

impl DataAccess {
    pub fn new(config: &Config) -> Result<DataAccess, Error> {
        Ok(DataAccess {
            pool: create_pool(config)?,
        })
    }

    pub fn with_transaction<F, T, E>(&self, f: F) -> Result<T, TxError<E>>
        where F : FnOnce(&Tx) -> Result<T, TxError<E>>,
              E : IntoTxError {
        let conn = self.pool.get()?;
        let tx = conn.transaction()?.into();
        let ret = f(&tx)?;
        tx.tx.commit()?;
        Ok(ret)
    }
}

pub struct Tx<'conn> {
    tx: Transaction<'conn>,
}

impl<'conn> From<Transaction<'conn>> for Tx<'conn> {
    fn from(tx: Transaction<'conn>) -> Tx<'conn> {
        Tx { tx }
    }
}

impl<'conn> Tx<'conn> {
    pub fn insert_api_key(&self, email: &str, prefix: &[u8], hashed_key: &[u8]) -> Result<Uuid, Error> {
        let id = Uuid::new_v4();
        self.tx.execute(
            "INSERT INTO api_key(id, email, prefix, hashed_key, created_dt) VALUES ($1, $2, $3, $4, NOW())",
            &[&id, &email, &prefix, &hashed_key])?;
        Ok(id)
    }

    pub fn email_exists(&self, email: &str) -> Result<bool, Error> {
        let rows = self.tx.query("SELECT COUNT(id) FROM api_key WHERE email = $1", &[&email])?;
        let count: i64 = rows.get(0).get_opt(0).unwrap()?;
        Ok(count > 0)
    }

    pub fn get_user_id_by_key(&self, prefix: &[u8], hashed_key: &[u8]) -> Result<Option<Uuid>, Error> {
        let rows = self.tx.query(
            "SELECT id FROM api_key WHERE prefix = $1 AND hashed_key = $2",
            &[&prefix, &hashed_key])?;
        if rows.len() == 0 {
            return Ok(None)
        }
        let first = rows.get(0);
        Ok(Some(first.get_opt(0).unwrap()?))
    }

    pub fn get_counter_by_tag_locking(&self, owner_id: Uuid, tag: &str) -> Result<Option<(Uuid, i64)>, Error> {
        let rows = self.tx.query(
            "SELECT id, counter_value FROM counter WHERE owner_id = $1 AND tag = $2 FOR UPDATE",
            &[&owner_id, &tag])?;
        if rows.len() == 0 {
            return Ok(None)
        }
        let first = rows.get(0);
        Ok(Some((first.get_opt(0).unwrap()?, first.get_opt(1).unwrap()?)))
    }

    pub fn create_counter(&self, owner_id: Uuid, tag: &str, initial: i64) -> Result<bool, Error> {
        let id = Uuid::new_v4();
        let rows_affected = self.tx.execute(
            "INSERT INTO counter(id, owner_id, tag, counter_value) VALUES ($1, $2, $3, $4) \
            ON CONFLICT DO NOTHING",
            &[&id, &owner_id, &tag, &initial])?;
        Ok(rows_affected > 0)
    }

    pub fn update_counter(&self, counter_id: Uuid, value: i64) -> Result<(), Error> {
        self.tx.execute(
            "UPDATE counter SET counter_value = $1, last_updated = NOW() WHERE id = $2",
            &[&value, &counter_id])?;
        Ok(())
    }
}

#[derive(Debug)]
pub enum Error {
    PostgresError(postgres::Error),
    R2D2Error(r2d2::Error),
}

impl From<postgres::Error> for Error {
    fn from(e: postgres::Error) -> Self {
        Error::PostgresError(e)
    }
}

impl From<r2d2::Error> for Error {
    fn from(e: r2d2::Error) -> Self {
        Error::R2D2Error(e)
    }
}

pub enum TxError<E> {
    DbError(Error),
    InnerError(E)
}

impl<E> From<postgres::Error> for TxError<E> {
    fn from(e: postgres::Error) -> Self {
        From::from(Error::PostgresError(e))
    }
}

impl<E> From<r2d2::Error> for TxError<E> {
    fn from(e: r2d2::Error) -> Self {
        From::from(Error::R2D2Error(e))
    }
}

impl<E> From<Error> for TxError<E> {
    fn from(e: Error) -> Self {
        TxError::DbError(e)
    }
}

impl<E : IntoTxError> From<E> for TxError<E> {
    fn from(e: E) -> Self {
        TxError::InnerError(e)
    }
}

pub trait IntoTxError {}

impl IntoTxError for () {}

pub struct Config {
    host: String,
    port: u16,
    database: String,
    username: String,
    password: String
}

impl Config {
    pub fn new(host: &str, port: u16, database: &str, username: &str, password: &str) -> Config {
        Config {
            host: host.to_string(),
            port,
            database: database.to_string(),
            username: username.to_string(),
            password: password.to_string()
        }
    }

    pub fn new_from_env() -> Option<Config> {
        use std::env::var;
        Some(Config {
            host: var("DB_HOST").ok()?,
            port: var("DB_PORT").ok()?.parse().ok()?,
            database: var("DB_NAME").ok()?,
            username: var("DB_USER").ok()?,
            password: var("DB_PASS").ok()?
        })
    }
}

fn create_pool(config: &Config) -> Result<Pool<PostgresConnectionManager>, Error> {
    log::info!("Creating connection pool...");
    let url = format!("postgresql://{}:{}@{}:{}/{}", config.username, config.password, config.host, config.port, config.database);
    let manager = PostgresConnectionManager::new(url, TlsMode::None)?;
    Pool::new(manager).map_err(|e| From::from(e))
}
