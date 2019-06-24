// use gio::{self, MenuExt};
use gtk::{self, prelude::*};

use crate::app::Action;

#[derive(Clone)]
pub struct HeaderBar {
    pub toggle_button: gtk::ToggleButton,
    pub switch_btn: gtk::Switch,
}

// Create headerbar for the application
//
// This includes the close button and in the future will include also various buttons
impl HeaderBar {
    pub fn new<P: gtk::GtkWindowExt>(window: &P) -> Self {
        let header_bar = gtk::HeaderBar::new();

        // Without this the headerbar will have no close button
        header_bar.set_show_close_button(true);

        // Create a menu button with the hamburger menu
        let main_menu = gtk::MenuButton::new();
        let main_menu_image =
            gtk::Image::new_from_icon_name("open-menu-symbolic", gtk::IconSize::Menu);
        main_menu.set_image(&main_menu_image);

        // Create a toggle button
        let toggle_button = gtk::ToggleButton::new();
        let toggle_button_image =
            gtk::Image::new_from_icon_name("camera-photo-symbolic", gtk::IconSize::Button);
        toggle_button.set_image(&toggle_button_image);
        // Place the button on the left
        // header_bar.pack_start(&toggle_button);

        // Create a switch button
        let switch_btn = gtk::Switch::new();
        // header_bar.pack_start(&switch_btn);

        // Create the menu model with the menu items. These directly activate our application
        // actions by their name
        let main_menu_model = gio::Menu::new();
        main_menu_model.append("About", Action::About.full_name());
        main_menu.set_menu_model(&main_menu_model);

        // And place it on the right (end) side of the header bar
        header_bar.pack_end(&main_menu);

        // Insert the headerbar as titlebar into the window
        window.set_titlebar(&header_bar);

        HeaderBar { toggle_button, switch_btn }
    }
}
