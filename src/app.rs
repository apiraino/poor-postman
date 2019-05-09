use gdk;
use gio::{self, prelude::*};
use glib;
use gtk::{self, prelude::*};

use std::cell::RefCell;
use std::error;
use std::ops;
use std::rc::{Rc, Weak};

// This represents our main application window.
#[derive(Clone)]
pub struct App(Rc<AppInner>);

// Deref into the contained struct to make usage a bit more ergonomic
impl ops::Deref for App {
    type Target = AppInner;

    fn deref(&self) -> &AppInner {
        &*self.0
    }
}

// App components
#[derive(Clone)]
pub struct AppInner {
    main_window: gtk::ApplicationWindow,
    // _header_bar: HeaderBar,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Action {
    Quit,
    // Settings,
    // About,
    // Snapshot(SnapshotState),
    // Record(RecordState),
}

impl App {
    fn new(application: &gtk::Application) -> Result<App, Box<dyn error::Error>> {
        // Here build the UI but don't show it yet
        let window = gtk::ApplicationWindow::new(application);

        window.set_title("GTK test");
        window.set_border_width(5);
        window.set_position(gtk::WindowPosition::Center);
        window.set_default_size(840, 480);

        // Create headerbar for the application window
        // let header_bar = HeaderBar::new(&window);

        let app = App(Rc::new(AppInner {
            main_window: window,
            // _header_bar: header_bar,
        }));

        // Create the application actions
        Action::create(&app, &application);

        Ok(app)
    }

    pub fn on_startup(application: &gtk::Application) {

        let app = match App::new(application) {
            Ok(app) => app,
            Err(err) => {
                eprintln!("Error creating app: {}",err);
                // utils::show_error_dialog (
                //     true,
                //     format!("Error creating application: {}", err).as_str(),
                // );
                return;
            }
        };

        application.connect_activate(clone!(app => move |_| {
            app.on_activate();
        }));

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
            // Action::Settings => "app.settings",
            // Action::About => "app.about",
            // Action::Snapshot(_) => "app.snapshot",
            // Action::Record(_) => "app.record",
        }
    }

    // Create our application actions here
    fn create(app: &App, application: &gtk::Application) {
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
