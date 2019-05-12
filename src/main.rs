extern crate gdk;
extern crate gio;
extern crate glib;
extern crate gtk;

use gio::prelude::*;

use std::env::args;
use std::error;

#[macro_use]
mod macros;
mod app;
mod utils;

use crate::app::App;

const APPLICATION_NAME : &str = "it.storiepvtride.gtk-test2";

fn main() -> Result<(), Box<dyn error::Error>> {

    let application = gtk::Application::new(APPLICATION_NAME, gio::ApplicationFlags::empty())?;
    application.connect_startup(|application| {
        App::on_startup(application);
    });
    application.run(&args().collect::<Vec<_>>());
    Ok(())
}
