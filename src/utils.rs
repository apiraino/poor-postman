use std::thread;

use reqwest::header::*;
use reqwest::Method;

use gtk::prelude::*;

use crate::http_client::{APIResponse, HttpClient};

pub fn spawn_thread(
    tx: &glib::Sender<String>,
    method: String,
    url: String,
    headers: Option<HeaderMap>,
    data: Option<serde_json::Value>,
) {
    eprintln!("spawing thread...");
    let verb =
        Method::from_bytes(method.as_bytes()).expect("Failed to decypher HTTP verb requested");

    // let mut headers = HeaderMap::new();
    // headers.insert(USER_AGENT, HeaderValue::from_static("poor-postman"));
    // headers.insert(ACCEPT, HeaderValue::from_static("application/json"));

    let _data: serde_json::Value = match data {
        Some(x) => x,
        None => json!({}),
    };

    let mut _headers = match headers {
        Some(x) => x,
        None => HeaderMap::new(),
    };

    thread::spawn(clone_old!(tx => move || {
        let client = HttpClient::new();
        eprintln!("headers: {:?}", _headers);
        match verb {
            Method::POST => {
                let resp_data = client.post(&url, _headers, &_data);
                let content = format_response(resp_data);
                // send result to channel
                tx.send(content)
                    .expect("Couldn't send data to channel");
            },
            Method::GET => {
                let resp_data = client.get(&url, _headers);
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

pub fn get_header_autocompletion(data: Vec<&str>, input_fld: &gtk::Entry) {
    let header_name_completion = gtk::EntryCompletion::new();
    let ls = create_list_model(data);
    header_name_completion.set_model(Some(&ls));
    input_fld.set_completion(Some(&header_name_completion));
    header_name_completion.set_text_column(0);
    header_name_completion.set_minimum_key_length(1);
    header_name_completion.set_popup_completion(true);
    input_fld.set_completion(Some(&header_name_completion));
}

fn format_response(response: Result<APIResponse, String>) -> String {
    match response {
        Ok(_response) => String::from(format!(
            "{}\n{}",
            _response.status_code,
            serde_json::to_string_pretty(&_response.data).expect("Failed to prettify string")
        )),
        Err(err_msg) => String::from(format!("Error: {}", err_msg)),
    }
}

pub fn compose_headers(headers_box: &gtk::Box) -> HeaderMap {
    let mut headers = HeaderMap::new();

    // let header_name_input = gtk::Entry::new();
    // let header_value_input = gtk::Entry::new();
    // headers_row.pack_start(&header_name_input, true, true, 10);
    // headers_row.pack_start(&header_value_input, true, true, 10);

    // upcasting && downcasting widgets
    // https://gtk-rs.org/docs-src/tutorial/upcast_downcast
    let children = headers_box.get_children();
    for child in children {
        let child_down = child.clone().downcast::<gtk::Entry>();
        if child_down.is_ok() {
            // eprintln!("{:#?}", child_down.unwrap());
            let txt = child_down.expect("this is safe").get_text().expect("this is safe");
            eprintln!("widget value {:?}", txt);
        }
    }

    headers.insert(USER_AGENT, HeaderValue::from_static("poor-postman"));
    headers
}

fn create_list_model(data: Vec<&str>) -> gtk::ListStore {
    let col_types: [gtk::Type; 1] = [gtk::Type::String];

    let store = gtk::ListStore::new(&col_types);
    let col_indices: [u32; 1] = [0];
    for d in data.iter() {
        let values: [&dyn ToValue; 1] = [&d];
        store.set(&store.append(), &col_indices, &values);
    }
    store
}
