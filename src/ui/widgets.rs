use adw::prelude::*;
use gtk::{gdk, glib, pango};

pub fn label(text: &str, classes: &[&str], xalign: f32, wrap: bool) -> gtk::Label {
    let widget = gtk::Label::new(Some(text));
    widget.set_xalign(xalign);
    widget.set_yalign(0.5);
    widget.set_wrap(wrap);
    if wrap {
        widget.set_wrap_mode(pango::WrapMode::WordChar);
    } else {
        widget.set_ellipsize(pango::EllipsizeMode::End);
    }
    for class in classes {
        widget.add_css_class(class);
    }
    widget
}

pub fn classed_button(text: &str, classes: &[&str]) -> gtk::Button {
    let widget = gtk::Button::with_label(text);
    for class in classes {
        widget.add_css_class(class);
    }
    widget
}

pub fn icon_button(icon_name: &str, classes: &[&str], tooltip: &str) -> gtk::Button {
    let widget = gtk::Button::new();
    let icon = gtk::Image::from_icon_name(icon_name);
    widget.set_child(Some(&icon));
    widget.set_tooltip_text(Some(tooltip));
    for class in classes {
        widget.add_css_class(class);
    }
    widget
}

pub fn clear_box(container: &gtk::Box) {
    while let Some(child) = container.first_child() {
        container.remove(&child);
    }
}

pub fn clear_grid(grid: &gtk::Grid) {
    while let Some(child) = grid.first_child() {
        grid.remove(&child);
    }
}

pub fn add_escape_to_close(window: &adw::ApplicationWindow) {
    let controller = gtk::EventControllerKey::new();
    let window_for_handler = window.clone();
    controller.connect_key_pressed(move |_, key, _, _| {
        if key == gdk::Key::Escape {
            window_for_handler.close();
            glib::Propagation::Stop
        } else {
            glib::Propagation::Proceed
        }
    });
    window.add_controller(controller);
}
