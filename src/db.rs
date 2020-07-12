use crate::{error, error::Error::*, DBCon, DBPool};
use mobc::async_trait;
use mobc::Pool;
use mobc_postgres::{tokio_postgres, PgConnectionManager};
use serde::Deserialize;
use std::fs;
use std::str::FromStr;
use tokio_postgres::{Config, Error, NoTls, Row};

type Result<T> = std::result::Result<T, error::Error>;

#[async_trait]
pub trait DBAccessor: Send + Sync + Clone + 'static {
    async fn fetch_todos(&self) -> Result<Vec<Todo>>;
    async fn create_todo(&self, name: String) -> Result<Todo>;
}

#[derive(Clone)]
pub struct DBAccess {
    pub db_pool: DBPool,
}

#[derive(Deserialize)]
pub struct Todo {
    pub id: i32,
    pub name: String,
    pub checked: bool,
}

const INIT_SQL: &str = "./db.sql";

pub fn create_pool() -> std::result::Result<DBPool, mobc::Error<Error>> {
    let config = Config::from_str("postgres://postgres@127.0.0.1:7878/postgres")?;
    Ok(Pool::builder().build(PgConnectionManager::new(config, NoTls)))
}

impl DBAccess {
    pub fn new(db_pool: DBPool) -> Self {
        Self { db_pool }
    }

    pub async fn init_db(&self) -> Result<()> {
        let init_file = fs::read_to_string(INIT_SQL)?;
        let con = self.get_db_con().await?;
        con.batch_execute(init_file.as_str())
            .await
            .map_err(DBInitError)?;
        Ok(())
    }

    async fn get_db_con(&self) -> Result<DBCon> {
        self.db_pool.get().await.map_err(DBPoolError)
    }

    fn row_to_todo(&self, row: &Row) -> Todo {
        let id: i32 = row.get(0);
        let name: String = row.get(1);
        let checked: bool = row.get(2);
        Todo { id, name, checked }
    }
}

#[async_trait]
impl DBAccessor for DBAccess {
    async fn fetch_todos(&self) -> Result<Vec<Todo>> {
        let con = self.get_db_con().await?;
        let query = "SELECT id, name, checked FROM todo ORDER BY id ASC";
        let q = con.query(query, &[]).await;
        let rows = q.map_err(DBQueryError)?;

        Ok(rows.iter().map(|r| self.row_to_todo(&r)).collect())
    }

    async fn create_todo(&self, name: String) -> Result<Todo> {
        let con = self.get_db_con().await?;
        let query = "INSERT INTO todo (name) VALUES ($1) RETURNING *";
        let row = con.query_one(query, &[&name]).await.map_err(DBQueryError)?;
        Ok(self.row_to_todo(&row))
    }
}
