#[macro_use]
#[cfg(test)]
extern crate lazy_static;

use http::HttpClient;
use mobc::{Connection, Pool};
use mobc_postgres::{tokio_postgres, PgConnectionManager};
use std::convert::Infallible;
use tokio_postgres::NoTls;
use warp::{Filter, Rejection, Reply};

mod db;
mod error;
mod handler;
mod http;
#[cfg(test)]
mod tests;

type Result<T> = std::result::Result<T, Rejection>;
type DBCon = Connection<PgConnectionManager<NoTls>>;
type DBPool = Pool<PgConnectionManager<NoTls>>;

#[tokio::main]
async fn main() {
    run().await;
}

async fn run() {
    let db_pool = db::create_pool().expect("database pool can be created");
    let db_access = db::DBAccess::new(db_pool);

    db_access
        .init_db()
        .await
        .expect("database can be initialized");
    let http_client = http::Client::new();

    println!("Server started at localhost:8080");
    warp::serve(router(http_client, db_access))
        .run(([0, 0, 0, 0], 8080))
        .await;
}

fn router(
    http_client: impl http::HttpClient,
    db_access: impl db::DBAccessor,
) -> impl Filter<Extract = impl Reply, Error = Infallible> + Clone {
    let todo = warp::path("todo");
    let todo_routes = todo
        .and(warp::get())
        .and(with_db(db_access.clone()))
        .and_then(handler::list_todos_handler)
        .or(todo
            .and(warp::post())
            .and(with_http_client(http_client.clone()))
            .and(with_db(db_access.clone()))
            .and_then(handler::create_todo));

    todo_routes.recover(error::handle_rejection)
}

fn with_db(
    db_access: impl db::DBAccessor,
) -> impl Filter<Extract = (impl db::DBAccessor,), Error = Infallible> + Clone {
    warp::any().map(move || db_access.clone())
}

fn with_http_client(
    http_client: impl HttpClient,
) -> impl Filter<Extract = (impl HttpClient,), Error = Infallible> + Clone {
    warp::any().map(move || http_client.clone())
}
