use crate::error;
use hyper::{body::to_bytes, client::HttpConnector, Body, Client as HyperClient, Method, Request};
use hyper_tls::HttpsConnector;
use mobc::async_trait;
use serde::Deserialize;
use serde_json::from_slice;

type Result<T> = std::result::Result<T, error::Error>;

const URI: &str = "https://cat-fact.herokuapp.com/facts/random";

#[async_trait]
pub trait HttpClient {
    async fn get_cat_fact(&self) -> Result<String>;
}

#[derive(Clone)]
pub struct Client {
    client: HyperClient<HttpsConnector<HttpConnector>>,
}

#[derive(Debug, Deserialize)]
pub struct CatFact {
    pub text: String,
}

impl Client {
    pub fn new() -> Self {
        let https = HttpsConnector::new();
        Self {
            client: HyperClient::builder().build::<_, Body>(https),
        }
    }
}

#[async_trait]
impl HttpClient for Client {
    async fn get_cat_fact(&self) -> Result<String> {
        let req = Request::builder()
            .method(Method::GET)
            .uri(URI)
            .header("content-type", "application/json")
            .header("accept", "application/json")
            .body(Body::empty())?;
        let res = self.client.request(req).await?;
        if !res.status().is_success() {
            return Err(error::Error::GetCatFactError(res.status()));
        }
        let body_bytes = to_bytes(res.into_body()).await?;
        let json = from_slice::<CatFact>(&body_bytes)?;
        Ok(json.text)
    }
}
