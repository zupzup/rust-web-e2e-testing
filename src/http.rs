use crate::error;
use hyper::{body::to_bytes, client::HttpConnector, Body, Client, Method, Request};
use hyper_tls::HttpsConnector;
use serde::Deserialize;
use serde_json::from_slice;

pub type HttpClient = Client<HttpsConnector<HttpConnector>>;
type Result<T> = std::result::Result<T, error::Error>;

const URI: &str = "https://cat-fact.herokuapp.com/facts/random";

#[derive(Debug, Deserialize)]
pub struct CatFact {
    pub text: String,
}

pub fn init_client() -> Client<HttpsConnector<HttpConnector>> {
    let https = HttpsConnector::new();
    Client::builder().build::<_, Body>(https)
}

pub async fn get_cat_fact(client: &HttpClient) -> Result<String> {
    let req = Request::builder()
        .method(Method::GET)
        .uri(URI)
        .header("content-type", "application/json")
        .header("accept", "application/json")
        .body(Body::empty())?;
    let res = client.request(req).await?;
    if !res.status().is_success() {
        return Err(error::Error::GetCatFactError(res.status()));
    }
    let body_bytes = to_bytes(res.into_body()).await?;
    let json = from_slice::<CatFact>(&body_bytes)?;
    Ok(json.text)
}
