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
            "INSERT INTO api_key(id, email, prefix, hashed_key, created_dt) VALUES ($1, $2, $3, $4, NOW());",
            &[&id, &email, &prefix, &hashed_key])?;
        Ok(id)
    }

    pub fn email_exists(&self, email: &str) -> Result<bool, Error> {
        let rows = self.tx.query("SELECT COUNT(id) FROM api_key WHERE email = $1", &[&email])?;
        Ok(rows.len() > 0)
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
}

fn create_pool(config: &Config) -> Result<Pool<PostgresConnectionManager>, Error> {
    let url = format!("postgresql://{}:{}@{}:{}/{}", config.username, config.password, config.host, config.port, config.database);
    let manager = PostgresConnectionManager::new(url, TlsMode::None)?;
    Pool::new(manager).map_err(|e| From::from(e))
}
