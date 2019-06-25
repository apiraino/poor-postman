use std::cell::RefCell;
use std::error;

use gio::{self, prelude::*};
use gtk::{self, prelude::*};

use crate::about_dialog::*;
use crate::header_bar::*;
use crate::utils::*;

#[derive(Clone)]
pub struct App {
    main_window: gtk::ApplicationWindow,
    pub header_bar: HeaderBar,
    url_input: gtk::Entry,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Action {
    About,
    Quit,
    ClickToggle(ToggleButtonState),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ToggleButtonState {
    State1,
    State2,
}

impl<'a> From<&'a glib::Variant> for ToggleButtonState {
    fn from(v: &glib::Variant) -> ToggleButtonState {
        v.get::<bool>().expect("Invalid record state type").into()
    }
}

impl From<bool> for ToggleButtonState {
    fn from(v: bool) -> ToggleButtonState {
        match v {
            false => ToggleButtonState::State1,
            true => ToggleButtonState::State2,
        }
    }
}

impl From<ToggleButtonState> for glib::Variant {
    fn from(v: ToggleButtonState) -> glib::Variant {
        match v {
            ToggleButtonState::State1 => false.to_variant(),
            ToggleButtonState::State2 => true.to_variant(),
        }
    }
}

trait GtkComboBoxTrait {
    fn get_text(self: &Self) -> String;
}

impl GtkComboBoxTrait for gtk::ComboBoxText {
    fn get_text(&self) -> String {
        self.get_active_text().expect("Failed to get widget text").to_string()
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
        let header_bar = HeaderBar::new(&main_window);

        // create a widget container,
        let layout = gtk::Box::new(gtk::Orientation::Vertical, 5);

        // Create a title label
        let url_title = gtk::Label::new(None);
        url_title.set_markup("<big>Type in your URL</big>");

        // Pressing Alt+T will activate this button
        let button = gtk::Button::new();
        let btn_label = gtk::Label::new_with_mnemonic(Some("_Click to trigger request"));
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
        verb_url_row.pack_start(&url_input, true, true, 10);

        // HTTP Headers
        let header_title = gtk::Label::new(None);
        header_title.set_markup("<big>Headers:</big>");

        let add_button = gtk::Button::new_with_label("Add");

        let header_name_input = gtk::Entry::new();
        let header_value_input = gtk::Entry::new();

        // TODO: Try to give a composite ID or name to retrieve them later
        // header_name_input.set_name("header_name_input");
        // header_value_input.set_name("header_value_input");

        // TODO: Dynamically add new items
        // https://github.com/gtk-rs/examples/blob/master/src/bin/listbox_model.rs
        // target/debug/listbox_model

        // Create a ListBox container
        let listbox = gtk::ListBox::new();

        // Add this "row"
        // https://paste.ubuntu.com/p/2TTkmBWdY4/

        // autocompletion for header names
        let data = vec!["Accept", "Authorization", "Content-Type"];
        // let mut count = 0;
        // for h in data.iter() {
        //     let _id: &str = &format!("ID{}", count);
        //     header_name_input.insert(count, Some(_id), h);
        //     count += 1;
        // }
        get_header_autocompletion(data, &header_name_input);

        // autocompletion for header values
        let data = vec!["application/json", "application/xml", "text/plain"];
        get_header_autocompletion(data, &header_value_input);

        let headers_row = gtk::Box::new(gtk::Orientation::Horizontal, 5);
        headers_row.add(&header_title);
        headers_row.add(&add_button);
        headers_row.pack_start(&header_name_input, true, true, 10);
        headers_row.pack_start(&header_value_input, true, true, 10);

        // Payload horizontal block
        let payload_title = gtk::Label::new(None);
        payload_title.set_markup("<big>Payload:</big>");
        let payload_input = gtk::Entry::new();
        payload_input.insert_text(r#"{"k": "key","v": "val"}"#, &mut 0);
        payload_input.set_sensitive(false);
        let payload_row = gtk::Box::new(gtk::Orientation::Horizontal, 5);
        payload_row.set_sensitive(false);
        payload_row.add(&payload_title);
        payload_row.pack_start(&payload_input, true, true, 0);

        // when POST is selected, activate the payload input box
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
        button.connect_clicked(clone!(
        button, verb_selector, url_input, payload_input, tx, headers_row => move |_| {
            button.set_sensitive(false);
            // and trigger HTTP thread
            spawn_thread(
                &tx,
                gtk::ComboBoxText::get_text(&verb_selector),
                url_input.get_buffer().get_text().to_owned(),
                // compose headers
                Some(compose_headers(&headers_row)),
                Some(json!(payload_input.get_buffer().get_text().to_owned()))
            );
        }));

        // connect the <Return> keypress to the callback
        url_input.connect_activate(clone!(
        button, verb_selector, payload_input, tx, headers_row => move |url_input_fld| {
            button.set_sensitive(false);
            spawn_thread(
                &tx,
                gtk::ComboBoxText::get_text(&verb_selector),
                url_input_fld.get_buffer().get_text().to_owned(),
                Some(compose_headers(&headers_row)),
                Some(json!(payload_input.get_buffer().get_text().to_owned()))
            );
        }));

        // connect Add button click
        add_button.connect_clicked(clone!(headers_row => move |btn| {
            eprintln!("Add button clicked: {:?}", btn);

            let header_key = gtk::Entry::new();
            let header_val = gtk::Entry::new();

            let new_row = gtk::Box::new(gtk::Orientation::Horizontal, 5);
            new_row.pack_start(&header_key, true, true, 10);
            new_row.pack_start(&header_val, true, true, 10);

            // TODO: Add this new "row" to the ListBox (somehow)

        }));

        // container for the HTTP response
        let response_container = gtk::TextView::new();
        response_container.set_editable(false);
        response_container.set_wrap_mode(gtk::WrapMode::Word);
        let buf = response_container.get_buffer().expect("I thought it could work...");
        buf.set_text("The response will appear here...");

        // add all widgets
        layout.add(&url_title);

        layout.add(&verb_url_row);
        layout.pack_start(&headers_row, false, false, 10);
        layout.pack_start(&payload_row, false, false, 10);
        layout.add(&trigger_btn_row);

        layout.pack_start(&response_container, true, true, 10);

        // add the widget container to the window
        main_window.add(&layout);

        let app = App { main_window, url_input, header_bar };

        // Create the application actions
        Action::create(&app, &application);

        // attach thread receiver
        rx.attach(None, move |text| {
            // let text = format_response(text);
            buf.set_text(&text);
            // enable the button again
            button.set_sensitive(true);
            // keep the channel open
            glib::Continue(true)
        });

        Ok(app)
    }

    pub fn on_startup(application: &gtk::Application) {
        let app = match App::new(application) {
            Ok(app) => app,
            Err(err) => {
                eprintln!("Error creating app: {}", err);
                return;
            }
        };

        application.connect_activate(clone!(app => move |_| {
            app.on_activate();
        }));

        // cant get rid of this RefCell wrapping ...
        let app_container = RefCell::new(Some(app));
        application.connect_shutdown(move |_| {
            let app = app_container.borrow_mut().take().expect("Shutdown called multiple times");
            app.on_shutdown();
        });
    }

    fn on_activate(&self) {
        // Show our window and bring it to the foreground
        self.main_window.show_all();
        self.main_window.present_with_time((glib::get_monotonic_time() / 1000) as u32);
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
            Action::About => "app.about",
            Action::Quit => "app.quit",
            Action::ClickToggle(_) => "app.toggle",
        }
    }

    // Create our application actions here
    fn create(app: &App, application: &gtk::Application) {
        eprintln!("Creating actions!");

        // about action: when activated it will show an about dialog
        let about = gio::SimpleAction::new("about", None);
        about.connect_activate(clone!(application => move |_action, _parameter| {
            show_about_dialog(&application);
        }));
        application.add_action(&about);

        // switch button action
        // credits: https://github.com/gtk-rs/examples/blob/master/src/bin/menu_bar_system.rs
        let switch_action = gio::SimpleAction::new_stateful("switch", None, &false.to_variant());
        let switch_btn = &app.header_bar.switch_btn;
        switch_btn.connect_property_active_notify(clone!(switch_action => move |s| {
            eprintln!("The switch is now {}", &s.get_active().to_variant());
            switch_action.change_state(&s.get_active().to_variant());
        }));
        application.add_action(&switch_action);

        // toggle button action
        let toggle_action = gio::SimpleAction::new_stateful("toggle", None, &false.to_variant());
        let toggle_btn = &app.header_bar.toggle_button;
        toggle_btn.connect_toggled(|btn| {
            eprintln!("Button state is {}", btn.get_active());
            let app = gio::Application::get_default().expect("No default application");
            Action::ClickToggle(ToggleButtonState::from(btn.get_active())).trigger(&app);
        });
        application.add_action(&toggle_action);

        // When activated, shuts down the application
        let quit = gio::SimpleAction::new("quit", None);
        quit.connect_activate(clone!(application => move |_action, _parameter| {
            application.quit();
        }));
        application.set_accels_for_action(Action::Quit.full_name(), &["<Primary>Q"]);
        application.add_action(&quit);
    }

    pub fn trigger<A: IsA<gio::Application> + gio::ActionGroupExt>(self, app: &A) {
        match self {
            Action::Quit => app.activate_action("quit", None),
            Action::About => app.activate_action("about", None),
            Action::ClickToggle(new_state) => app.change_action_state("toggle", &new_state.into()),
        }
    }
}
