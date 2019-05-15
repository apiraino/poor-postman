use reqwest::{Client, StatusCode};
use reqwest::Method;
use reqwest::header::*;
use serde_derive::Deserialize;
use serde_json::Value;

pub struct HttpClient {
    client: Client
}

#[derive(Debug, Deserialize)]
pub struct APIResponse {
    pub status_code: u16,
    pub data: Value
}

impl HttpClient {

    pub fn new() -> HttpClient {
        HttpClient {
            client: Client::new(),
        }
    }

    pub fn post(&self, url: &str, headers: HeaderMap, data: &Value) -> APIResponse {
        let mut resp = self.client
            .request(Method::POST, url)
            .headers(headers)
            .json(&data)
            .send()
            .expect("POST failed");
        assert_eq!(resp.status() < StatusCode::INTERNAL_SERVER_ERROR,
                   true);
        APIResponse {
            status_code: resp.status().as_u16(),
            data: resp.json().unwrap_or(
                json!({
                    "status": resp.status().as_u16(),
                    "detail": "Failed to deserialize response"
                }))
        }
    }

    pub fn get(&self, url: &str, headers: HeaderMap) -> APIResponse {
        let mut resp = self.client
            .request(Method::GET, url)
            .headers(headers)
            .send()
            .expect("GET failed");
        assert_eq!(resp.status() < StatusCode::INTERNAL_SERVER_ERROR,
                   true);
        APIResponse {
            status_code: resp.status().as_u16(),
            data: resp.json().unwrap_or(
                json!({
                    "status": resp.status().as_u16(),
                    "detail": "Failed to deserialize response"
                }))
        }
    }
}
