use std::cell::RefCell;
use std::error;

use gio::{self, prelude::*};
use gtk::{self, prelude::*};

use crate::utils::*;

#[derive(Clone)]
pub struct App {
    main_window: gtk::ApplicationWindow,
    url_input: gtk::Entry
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Action {
    Quit,
}

trait GtkComboBoxTrait {
    fn get_text(self: &Self) -> String;
}

impl GtkComboBoxTrait for gtk::ComboBoxText {
    fn get_text(&self) -> String {
       let s = self
            .get_active_text()
            .expect("Failed to get widget text")
            .to_string();
        s.clone()
    }
}

impl App {
    fn new(application: &gtk::Application) -> Result<App, Box<dyn error::Error>> {

        let (tx, rx) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);

        // Here build the UI but don't show it yet
        let main_window = gtk::ApplicationWindow::new(application);
        main_window.set_title("(poor) Postman");
        main_window.set_border_width(5);
        main_window.set_position(gtk::WindowPosition::Center);
        main_window.set_default_size(840, 480);

        // Create headerbar for the application window
        // let header_bar = HeaderBar::new(&window);

        // create a widget container,
        let layout = gtk::Box::new(gtk::Orientation::Vertical, 5);

        // Create a title label
        let url_title = gtk::Label::new(None);
        url_title.set_markup("<big>Type in your URL</big>");

        // Pressing Alt+T will activate this button
        let button = gtk::Button::new();
        let btn_label = gtk::Label::new_with_mnemonic(
            Some("_Click to trigger request")
        );
        button.add(&btn_label);

        // Trigger request button
        let trigger_btn_row = gtk::Box::new(gtk::Orientation::Horizontal, 5);
        trigger_btn_row.pack_start(&button, false, true, 10);

        let url_input = gtk::Entry::new();
        url_input.set_placeholder_text("(poor) Postman");
        url_input.insert_text("http://httpbin.org/get", &mut 0);

        let verb_selector = gtk::ComboBoxText::new();
        verb_selector.insert(0, "ID0", "GET");
        verb_selector.insert(1, "ID1", "POST");
        verb_selector.set_active(Some(0));

        let verb_url_row = gtk::Box::new(gtk::Orientation::Horizontal, 5);
        verb_url_row.add(&verb_selector);
        // http://gtk-rs.org/docs/gtk/prelude/trait.BoxExt.html#tymethod.pack_start
        // params: child, expand, fill, padding (px)
        verb_url_row.pack_start(&url_input, true, true, 0);

        // Payload horizontal block
        let payload_title = gtk::Label::new(None);
        payload_title.set_markup("<big>Payload</big>");
        let payload_input = gtk::Entry::new();
        payload_input.insert_text(r#"ex. {"k": "key","v": "val"}"#, &mut 0);
        payload_input.set_sensitive(false);
        let payload_row = gtk::Box::new(gtk::Orientation::Horizontal, 5);
        payload_row.set_sensitive(false);
        payload_row.add(&payload_title);
        payload_row.pack_start(&payload_input, true, true, 0);

        // when POST is selected, activate the payload input box
        // TODO: why don't I need to also clone "payload_input"?
        verb_selector.connect_changed(clone!(payload_row, payload_input => move |verb_selector| {
            let txt = gtk::ComboBoxText::get_text(&verb_selector);
            match txt.as_ref() {
                "POST" => {
                    payload_row.set_sensitive(true);
                    payload_input.set_sensitive(true);
                }
                _ => {
                    payload_row.set_sensitive(false);
                    payload_input.set_sensitive(false);
                }
            }
        }));

        // connect the Button click to the callback
        button.connect_clicked(clone!(button, verb_selector, url_input,
                                      payload_input, tx => move |_| {
            button.set_sensitive(false);
            // and trigger HTTP thread
            spawn_thread(
                &tx,
                gtk::ComboBoxText::get_text(&verb_selector),
                url_input.get_buffer().get_text().to_owned(),
                Some(json!(payload_input.get_buffer().get_text().to_owned()))
            );
        }));

        // connect the <Return> keypress to the callback
        url_input.connect_activate(clone!(button, verb_selector,
                                          payload_input, tx => move |_entry| {
            button.set_sensitive(false);
            spawn_thread(
                &tx,
                gtk::ComboBoxText::get_text(&verb_selector),
                _entry.get_buffer().get_text().to_owned(),
                Some(json!(payload_input.get_buffer().get_text().to_owned()))
            );
        }));

        // container for the response
        let response_container = gtk::TextView::new();
        response_container.set_editable(false);
        response_container.set_wrap_mode(gtk::WrapMode::Word);
        let buf = response_container.get_buffer().expect("I thought it could work...");
        buf.set_text("The response will appear here...");

        // add all widgets
        layout.add(&url_title);
        layout.add(&verb_url_row);
        layout.pack_start(&payload_row, false, true, 10);
        layout.add(&trigger_btn_row);
        layout.pack_start(&response_container, true, true, 10);

        // add the widget container to the window
        main_window.add(&layout);

        let app = App {
            main_window,
            url_input
            // _header_bar: header_bar,
        };

        // Create the application actions
        Action::create(&app, &application);

        // attach thread receiver
        rx.attach(None, move |text| {
            // let text = format_response(text);
            buf.set_text(&text);
            // enable the button again
            button.set_sensitive(true);
            // keeps the channel open
            glib::Continue(true)
        });

        Ok(app)
    }

    pub fn on_startup(application: &gtk::Application) {

        let app = match App::new(application) {
            Ok(app) => app,
            Err(err) => {
                eprintln!("Error creating app: {}",err);
                return;
            }
        };

        application.connect_activate(clone!(app => move |_| {
            app.on_activate();
        }));

        // cant get rid of this RefCell wrapping ...
        let app_container = RefCell::new(Some(app));
        application.connect_shutdown(move |_| {
            let app = app_container
                .borrow_mut()
                .take()
                .expect("Shutdown called multiple times");
            app.on_shutdown();
        });
    }

    fn on_activate(&self) {
        // Show our window and bring it to the foreground
        self.main_window.show_all();
        self.main_window
            .present_with_time((glib::get_monotonic_time() / 1000) as u32);
    }

    // Called when the application shuts down. We drop our app struct here
    fn on_shutdown(self) {
        eprintln!("Shutting down the whole thing");
    }
}

impl Action {

    // The full action name as is used in e.g. menu models
    pub fn full_name(self) -> &'static str {
        match self {
            Action::Quit => "app.quit",
        }
    }

    // Create our application actions here
    fn create(_app: &App, application: &gtk::Application) {
        eprintln!("Creating actions!");

        // When activated, shuts down the application
        let quit = gio::SimpleAction::new("quit", None);
        quit.connect_activate(clone!(application => move |_action, _parameter| {
            application.quit();
        }));
        application.set_accels_for_action(Action::Quit.full_name(), &["<Primary>Q"]);
        application.add_action(&quit);
    }
}
