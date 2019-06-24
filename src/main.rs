extern crate gdk;
extern crate gio;
extern crate glib;
extern crate gtk;

use gio::prelude::*;

use std::env::args;
use std::error;

#[macro_use]
extern crate serde_json;

#[macro_use]
mod macros;
mod about_dialog;
mod app;
mod header_bar;
mod http_client;
mod utils;

use crate::app::App;

const APPLICATION_NAME: &str = "apiraino.poor.postman";

fn main() -> Result<(), Box<dyn error::Error>> {
    let application = gtk::Application::new(APPLICATION_NAME, gio::ApplicationFlags::empty())?;
    application.connect_startup(|application| {
        App::on_startup(application);
    });
    application.run(&args().collect::<Vec<_>>());
    Ok(())
}
