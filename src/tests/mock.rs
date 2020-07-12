use crate::db::{DBAccessor, Todo};
use crate::http::HttpClient;
use crate::{error, run};
use mobc::async_trait;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;
use tokio::time::delay_for;
use wiremock::{
    matchers::{method, path},
    Mock, MockServer, ResponseTemplate,
};

#[derive(Clone)]
pub struct MockHttpClient {}
type Result<T> = std::result::Result<T, error::Error>;

#[async_trait]
impl HttpClient for MockHttpClient {
    async fn get_cat_fact(&self) -> Result<String> {
        Ok(String::from("cat fact"))
    }
}

#[derive(Clone)]
pub struct MockDBAccessor {}

#[async_trait]
impl DBAccessor for MockDBAccessor {
    async fn fetch_todos(&self) -> Result<Vec<Todo>> {
        Ok(vec![Todo {
            id: 1,
            name: String::from("first todo"),
            checked: true,
        }])
    }

    async fn create_todo(&self, name: String) -> Result<Todo> {
        Ok(Todo {
            id: 2,
            name: name,
            checked: false,
        })
    }
}

pub struct WiremockServer {
    pub server: Option<MockServer>,
}

impl WiremockServer {
    pub fn new() -> Self {
        Self { server: None }
    }

    pub async fn init(&mut self) {
        let mock_server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/facts/random"))
            .respond_with(
                ResponseTemplate::new(200).set_body_string(r#"{"text": "wiremock cat fact"}"#),
            )
            .mount(&mock_server)
            .await;
        self.server = Some(mock_server);
    }
}

pub struct Server {
    pub started: AtomicBool,
}

impl Server {
    pub fn new() -> Server {
        Server {
            started: AtomicBool::new(false),
        }
    }

    pub async fn init_server(&mut self) {
        if !self.started.load(Ordering::Relaxed) {
            thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().expect("runtime starts");
                rt.spawn(run());
                loop {
                    thread::sleep(Duration::from_millis(100_000));
                }
            });
            delay_for(Duration::from_millis(100)).await;
            self.started.store(true, Ordering::Relaxed);
        }
    }
}
