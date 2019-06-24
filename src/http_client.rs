use reqwest::header::*;
use reqwest::Method;
use reqwest::{Client, StatusCode};
use serde_derive::Deserialize;
use serde_json::Value;
use std::error::Error;

pub struct HttpClient {
    client: Client,
}

#[derive(Debug, Deserialize)]
pub struct APIResponse {
    pub status_code: u16,
    pub data: Value,
}

impl HttpClient {
    pub fn new() -> HttpClient {
        HttpClient { client: Client::new() }
    }

    pub fn post(&self, url: &str, headers: HeaderMap, data: &Value) -> Result<APIResponse, String> {
        let req = self.client.request(Method::POST, url).headers(headers).json(&data).send();
        let mut resp = match req {
            Ok(x) => x,
            Err(err) => {
                let msg = self.handler(err);
                eprintln!("Could not perform request: {:?}", msg);
                return Err(msg);
            }
        };

        assert_eq!(resp.status() < StatusCode::INTERNAL_SERVER_ERROR, true);
        Ok(APIResponse {
            status_code: resp.status().as_u16(),
            data: resp.json().unwrap_or(json!({
                "status": resp.status().as_u16(),
                "detail": "Failed to deserialize response"
            })),
        })
    }

    pub fn get(&self, url: &str, headers: HeaderMap) -> Result<APIResponse, String> {
        let req = self.client.request(Method::GET, url).headers(headers).send();

        let mut resp = match req {
            Ok(x) => x,
            Err(err) => {
                let msg = self.handler(err);
                // let msg = format!("Could not perform request: {:?}", msg);
                eprintln!("{}", msg);
                return Err(msg);
            }
        };

        assert_eq!(resp.status() < StatusCode::INTERNAL_SERVER_ERROR, true);
        Ok(APIResponse {
            status_code: resp.status().as_u16(),
            data: resp.json().unwrap_or(json!({
                "status": resp.status().as_u16(),
                "detail": "Failed to deserialize response"
            })),
        })
    }

    fn handler(&self, e: reqwest::Error) -> String {
        if e.is_http() {
            let url_error = match e.url() {
                None => format!("{}", "No Url given"),
                Some(url) => format!("Problem requesting url: {}", url),
            };
            return url_error;
        }
        // Inspect the internal error and output it
        if e.is_serialization() {
            let serde_error = match e.get_ref() {
                None => "No description available",
                Some(err) => err.description(),
            };
            let err = format!("problem parsing information {}", serde_error);
            return err.to_owned();
        }
        if e.is_redirect() {
            return "Server redirecting too many times or making loop".to_owned();
        }
        e.description().to_owned()
    }
}
