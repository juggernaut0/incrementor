use r2d2_postgres::{PostgresConnectionManager, TlsMode};
use r2d2::Pool;

pub struct DataAccess {
    pool: Pool<PostgresConnectionManager>,
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

impl DataAccess {
    pub fn new(config: &Config) -> Result<DataAccess, Error> {
        Ok(DataAccess {
            pool: create_pool(config)?,
        })
    }

    pub fn test_call(&self) {
        let conn = self.pool.get().unwrap();
        let rows = conn.query("SELECT 'hello world'", &[]).unwrap();
        let value: String = rows.get(0).get(0);
        println!("test_call: result = {}", value);
    }
}

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
