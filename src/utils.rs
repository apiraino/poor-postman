use std::thread;

// use crate::http_client::HttpClient;

use reqwest::Client;
use reqwest::Method;
use reqwest::header::*;
use serde_derive::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
pub struct APIResponse {
    status_code: u16,
    data: Value
}

// TODO: use str slice
pub fn spawn_thread(tx: &glib::Sender<String>, method: String, url: String) {
    eprintln!("spawing thread...");
    let client = Client::new();
    let verb = Method::from_bytes(method.as_bytes())
        .expect("Failed to decypher HTTP verb requested");
    eprintln!("Method will be {:?}", verb);

    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static("poor-postman"));
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert(ACCEPT, HeaderValue::from_static("application/json"));

    // TODO work around these values moved
    thread::spawn(clone_old!(tx => move || {

        match verb {
            Method::POST => {
                let req = client
                    .request(verb, &url)
                    .headers(headers);
                let data = json!({
                    "key": "value"
                });
                let mut resp = req
                    .json(&data)
                    .send()
                    .expect("Request failed");
                eprintln!("resp data: {:?}", resp);
                let response = APIResponse {
                    status_code: resp.status().as_u16(),
                    data: resp.json().unwrap_or(
                        json!({
                            "status": resp.status().as_u16(),
                            "detail": "Failed to deserialize response"
                        }))
                };
                let content = format_response(response);
                // send result to channel
                tx.send(content)
                    .expect("Couldn't send data to channel");
            },
            Method::GET => {
                let req = client
                    .request(verb, &url)
                    .headers(headers);
                let mut resp = req
                    // .json(&data)
                    .send()
                    .expect("Request failed");
                eprintln!("resp data: {:?}", resp);
                let response = APIResponse {
                    status_code: resp.status().as_u16(),
                    data: resp.json().unwrap_or(
                        json!({
                            "status": resp.status().as_u16(),
                            "detail": "Failed to deserialize response"
                        }))
                };
                let content = format_response(response);
                // send result to channel
                tx.send(content)
                    .expect("Couldn't send data to channel");
            },
            _ => {
                eprintln!("Not implemented yet");
            }
        }

    }));
}

fn format_response(response: APIResponse) -> String {
    String::from(format!("{}\n{}", response.status_code, response.data))
}
