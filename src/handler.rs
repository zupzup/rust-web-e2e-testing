use crate::{
    db::{DBAccessor, Todo},
    http::HttpClient,
    Result,
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

pub async fn list_todos_handler(db_access: impl DBAccessor) -> Result<impl Reply> {
    let todos = db_access
        .fetch_todos()
        .await
        .map_err(|e| reject::custom(e))?;
    Ok(json::<Vec<_>>(
        &todos.into_iter().map(|t| TodoResponse::of(t)).collect(),
    ))
}

pub async fn create_todo(
    http_client: impl HttpClient,
    db_access: impl DBAccessor,
) -> Result<impl Reply> {
    let cat_fact = http_client
        .get_cat_fact()
        .await
        .map_err(|e| reject::custom(e))?;
    Ok(json(&TodoResponse::of(
        db_access
            .create_todo(cat_fact)
            .await
            .map_err(|e| reject::custom(e))?,
    )))
}
