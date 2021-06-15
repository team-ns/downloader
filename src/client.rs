use anyhow::Result;
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::{header, Client};

pub fn get_keep_alive_client() -> Result<Client> {
    let mut headers = HeaderMap::new();
    headers.insert(header::CONNECTION, HeaderValue::from_static("keep-alive"));
    Ok(Client::builder().default_headers(headers).build()?)
}
