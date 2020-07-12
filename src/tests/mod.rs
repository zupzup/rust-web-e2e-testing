use crate::{db, http, router};
use hyper::{body::to_bytes, client::HttpConnector, Body, Client as HyperClient, Method, Request};
use hyper_tls::HttpsConnector;
use mock::*;
use std::sync::RwLock;
use warp::test::request;

mod mock;

lazy_static! {
    pub static ref MOCK_HTTP_SERVER: RwLock<WiremockServer> = RwLock::new(WiremockServer::new());
    static ref SERVER: RwLock<Server> = RwLock::new(Server::new());
}

// Pure Mock Tests with HTTP client and DB mocked
#[tokio::test]
async fn test_health_mock() {
    let r = router(MockHttpClient {}, MockDBAccessor {});
    let resp = request().path("/health").reply(&r).await;
    assert_eq!(resp.status(), 200);
    assert_eq!(resp.body(), "OK");
}

#[tokio::test]
async fn test_list_todos_mock() {
    let r = router(MockHttpClient {}, MockDBAccessor {});
    let resp = request().path("/todo").reply(&r).await;
    assert_eq!(resp.status(), 200);
    assert_eq!(
        resp.body(),
        r#"[{"id":1,"name":"first todo","checked":true}]"#
    );
}

#[tokio::test]
async fn test_create_todo_mock() {
    let r = router(MockHttpClient {}, MockDBAccessor {});
    let resp = request()
        .path("/todo")
        .method("POST")
        .body("")
        .reply(&r)
        .await;
    assert_eq!(resp.status(), 200);
    assert_eq!(resp.body(), r#"{"id":2,"name":"cat fact","checked":false}"#);
}

// Hybrid Mock Tests with Mock DB and Wiremock
#[tokio::test]
async fn test_create_and_list_todo_hybrid() {
    setup_wiremock().await;
    let r = router(http::Client::new(), MockDBAccessor {});
    let resp = request()
        .path("/todo")
        .method("POST")
        .body("")
        .reply(&r)
        .await;
    assert_eq!(resp.status(), 200);
    assert_eq!(
        resp.body(),
        r#"{"id":2,"name":"wiremock cat fact","checked":false}"#
    );

    let resp = request().path("/todo").reply(&r).await;
    assert_eq!(resp.status(), 200);
    assert_eq!(
        resp.body(),
        r#"[{"id":1,"name":"first todo","checked":true}]"#
    );
}

async fn setup_wiremock() {
    MOCK_HTTP_SERVER.write().unwrap().init().await;
}

// Full Test with Wiremock and real DB
#[tokio::test]
async fn test_create_and_list_full() {
    setup_wiremock().await;
    let r = router(http::Client::new(), init_db().await);
    let resp = request()
        .path("/todo")
        .method("POST")
        .body("")
        .reply(&r)
        .await;
    assert_eq!(resp.status(), 200);
    assert_eq!(
        resp.body(),
        r#"{"id":1,"name":"wiremock cat fact","checked":false}"#
    );

    let resp = request().path("/todo").reply(&r).await;
    assert_eq!(resp.status(), 200);
    assert_eq!(
        resp.body(),
        r#"[{"id":1,"name":"wiremock cat fact","checked":false}]"#
    );
}

async fn init_db() -> impl db::DBAccessor {
    let db_pool = db::create_pool().expect("database pool can be created");
    let db_access = db::DBAccess::new(db_pool.clone());

    db_access
        .init_db()
        .await
        .expect("database can be initialized");

    let con = db_pool.get().await.unwrap();
    let query = format!("BEGIN;DELETE FROM todo;ALTER SEQUENCE todo_id_seq RESTART with 1;COMMIT;");
    let _ = con.batch_execute(query.as_str()).await;

    db_access
}

// E2E Tests with actual server, Wiremock and DB
#[tokio::test]
async fn test_create_and_list_e2e() {
    setup_wiremock().await;
    init_real_server().await;
    let http_client = http_client();

    let req = Request::builder()
        .method(Method::POST)
        .uri("http://localhost:8080/todo")
        .body(Body::empty())
        .unwrap();
    let resp = http_client.request(req).await.unwrap();
    assert_eq!(resp.status(), 200);
    let body_bytes = to_bytes(resp.into_body()).await.unwrap();
    assert_eq!(
        body_bytes,
        r#"{"id":1,"name":"wiremock cat fact","checked":false}"#
    );

    let req = Request::builder()
        .method(Method::GET)
        .uri("http://localhost:8080/todo")
        .body(Body::empty())
        .unwrap();
    let resp = http_client.request(req).await.unwrap();
    assert_eq!(resp.status(), 200);
    let body_bytes = to_bytes(resp.into_body()).await.unwrap();
    assert_eq!(
        body_bytes,
        r#"[{"id":1,"name":"wiremock cat fact","checked":false}]"#
    );
}

#[tokio::test]
async fn test_list_e2e() {
    setup_wiremock().await;
    init_real_server().await;
    let http_client = http_client();

    let req = Request::builder()
        .method(Method::GET)
        .uri("http://localhost:8080/todo")
        .body(Body::empty())
        .unwrap();
    let resp = http_client.request(req).await.unwrap();
    assert_eq!(resp.status(), 200);
    let body_bytes = to_bytes(resp.into_body()).await.unwrap();
    assert_eq!(body_bytes, r#"[]"#);
}

async fn init_real_server() {
    let _ = init_db().await;
    SERVER.write().unwrap().init_server().await;
}

fn http_client() -> HyperClient<HttpsConnector<HttpConnector>> {
    let https = HttpsConnector::new();
    HyperClient::builder().build::<_, Body>(https)
}
