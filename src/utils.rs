use std::thread;

use reqwest::Method;
use reqwest::header::*;

use crate::http_client::{APIResponse, HttpClient};

pub fn spawn_thread(tx: &glib::Sender<String>,
                    method: String,
                    url: String,
                    data: Option<serde_json::Value>) {
    eprintln!("spawing thread...");
    let verb = Method::from_bytes(method.as_bytes())
        .expect("Failed to decypher HTTP verb requested");

    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static("poor-postman"));
    headers.insert(ACCEPT, HeaderValue::from_static("application/json"));

    let _data : serde_json::Value = match data {
        Some(x) => x,
        None => json!({})
    };

    thread::spawn(clone_old!(tx => move || {
        let client = HttpClient::new();
        match verb {
            Method::POST => {
                headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
                let resp_data = client.post(&url, headers, &_data);
                let content = format_response(resp_data);
                // send result to channel
                tx.send(content)
                    .expect("Couldn't send data to channel");
            },
            Method::GET => {
                let resp_data = client.get(&url, headers);
                let content = format_response(resp_data);
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
    String::from(format!("{}\n{}",
                         response.status_code,
                         serde_json::to_string_pretty(&response.data)
                             .expect("Failed to prettify string")
    ))
}
