use crate::{
    db::{self, Todo},
    http::{get_cat_fact, HttpClient},
    DBPool, Result,
};
use serde::Serialize;
use warp::{reject, reply::json, Reply};

#[derive(Serialize)]
pub struct TodoResponse {
    pub id: i32,
    pub name: String,
    pub checked: bool,
}

impl TodoResponse {
    pub fn of(todo: Todo) -> TodoResponse {
        TodoResponse {
            id: todo.id,
            name: todo.name,
            checked: todo.checked,
        }
    }
}

pub async fn health_handler() -> Result<impl Reply> {
    Ok("OK")
}

pub async fn list_todos_handler(db_pool: DBPool) -> Result<impl Reply> {
    let todos = db::fetch_todos(&db_pool)
        .await
        .map_err(|e| reject::custom(e))?;
    Ok(json::<Vec<_>>(
        &todos.into_iter().map(|t| TodoResponse::of(t)).collect(),
    ))
}

pub async fn create_todo(http_client: HttpClient, db_pool: DBPool) -> Result<impl Reply> {
    let cat_fact = get_cat_fact(&http_client)
        .await
        .map_err(|e| reject::custom(e))?;
    Ok(json(&TodoResponse::of(
        db::create_todo(&db_pool, cat_fact)
            .await
            .map_err(|e| reject::custom(e))?,
    )))
}
