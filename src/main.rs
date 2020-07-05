use http::HttpClient;
use mobc::{Connection, Pool};
use mobc_postgres::{tokio_postgres, PgConnectionManager};
use std::convert::Infallible;
use tokio_postgres::NoTls;
use warp::{Filter, Rejection};

mod db;
mod error;
mod handler;
mod http;

type Result<T> = std::result::Result<T, Rejection>;
type DBCon = Connection<PgConnectionManager<NoTls>>;
type DBPool = Pool<PgConnectionManager<NoTls>>;

#[tokio::main]
async fn main() {
    let db_pool = db::create_pool().expect("database pool can be created");

    db::init_db(&db_pool)
        .await
        .expect("database can be initialized");

    run(db_pool).await;
}

async fn run(db_pool: DBPool) {
    let http_client = http::init_client();
    let health_route = warp::path("health")
        .and(warp::get())
        .and_then(handler::health_handler);
    let todo = warp::path("todo");
    let todo_routes = todo
        .and(warp::get())
        .and(with_db(db_pool.clone()))
        .and_then(handler::list_todos_handler)
        .or(todo
            .and(warp::post())
            .and(with_http_client(http_client.clone()))
            .and(with_db(db_pool.clone()))
            .and_then(handler::create_todo));

    let routes = todo_routes
        .or(health_route)
        .recover(error::handle_rejection);

    println!("Server started at localhost:8080");
    warp::serve(routes).run(([0, 0, 0, 0], 8080)).await;
}

fn with_db(db_pool: DBPool) -> impl Filter<Extract = (DBPool,), Error = Infallible> + Clone {
    warp::any().map(move || db_pool.clone())
}

fn with_http_client(
    http_client: HttpClient,
) -> impl Filter<Extract = (HttpClient,), Error = Infallible> + Clone {
    warp::any().map(move || http_client.clone())
}
