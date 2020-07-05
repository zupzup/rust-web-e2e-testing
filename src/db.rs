use crate::{error, error::Error::*, DBCon, DBPool};
use mobc::Pool;
use mobc_postgres::{tokio_postgres, PgConnectionManager};
use serde::Deserialize;
use std::fs;
use std::str::FromStr;
use tokio_postgres::{Config, Error, NoTls, Row};

type Result<T> = std::result::Result<T, error::Error>;

#[derive(Deserialize)]
pub struct Todo {
    pub id: i32,
    pub name: String,
    pub checked: bool,
}

const INIT_SQL: &str = "./db.sql";

pub async fn init_db(db_pool: &DBPool) -> Result<()> {
    let init_file = fs::read_to_string(INIT_SQL)?;
    let con = get_db_con(db_pool).await?;
    con.batch_execute(init_file.as_str())
        .await
        .map_err(DBInitError)?;
    Ok(())
}

pub async fn get_db_con(db_pool: &DBPool) -> Result<DBCon> {
    db_pool.get().await.map_err(DBPoolError)
}

pub fn create_pool() -> std::result::Result<DBPool, mobc::Error<Error>> {
    let config = Config::from_str("postgres://postgres@127.0.0.1:7878/postgres")?;
    Ok(Pool::builder().build(PgConnectionManager::new(config, NoTls)))
}

pub async fn fetch_todos(db_pool: &DBPool) -> Result<Vec<Todo>> {
    let con = get_db_con(db_pool).await?;
    let query = "SELECT id, name, checked FROM todo ORDER BY id ASC";
    let q = con.query(query, &[]).await;
    let rows = q.map_err(DBQueryError)?;

    Ok(rows.iter().map(|r| row_to_todo(&r)).collect())
}

pub async fn create_todo(db_pool: &DBPool, name: String) -> Result<Todo> {
    let con = get_db_con(db_pool).await?;
    let query = "INSERT INTO todo (name) VALUES ($1) RETURNING *";
    let row = con.query_one(query, &[&name]).await.map_err(DBQueryError)?;
    Ok(row_to_todo(&row))
}

fn row_to_todo(row: &Row) -> Todo {
    let id: i32 = row.get(0);
    let name: String = row.get(1);
    let checked: bool = row.get(2);
    Todo { id, name, checked }
}
