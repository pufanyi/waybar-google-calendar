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
    widget.set_cursor_from_name(Some("pointer"));
    for class in classes {
        widget.add_css_class(class);
    }
    widget
}

pub fn icon_button(icon_name: &str, classes: &[&str], tooltip: &str) -> gtk::Button {
    let widget = gtk::Button::new();
    widget.set_cursor_from_name(Some("pointer"));
    let icon = gtk::Image::from_icon_name(icon_name);
    widget.set_child(Some(&icon));
    widget.set_tooltip_text(Some(tooltip));
    for class in classes {
        widget.add_css_class(class);
    }
    widget
}

pub(super) fn block_scroll_changes(widget: &impl IsA<gtk::Widget>) {
    let scroll = gtk::EventControllerScroll::new(gtk::EventControllerScrollFlags::BOTH_AXES);
    scroll.set_propagation_phase(gtk::PropagationPhase::Capture);
    scroll.connect_scroll(|_, _, _| glib::Propagation::Stop);
    widget.add_controller(scroll);
}

pub fn drop_down() -> gtk::DropDown {
    let widget = gtk::DropDown::from_strings(&[]);
    block_scroll_changes(&widget);
    widget
}

pub fn set_drop_down_strings(widget: &gtk::DropDown, labels: &[&str], selected: usize) {
    let model = gtk::StringList::new(labels);
    widget.set_model(Some(&model));
    let selected = if labels.is_empty() {
        gtk::INVALID_LIST_POSITION
    } else {
        selected.min(labels.len() - 1) as u32
    };
    widget.set_selected(selected);
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

pub fn add_escape_action<F>(window: &adw::ApplicationWindow, on_escape: F)
where
    F: Fn() + 'static,
{
    let controller = gtk::EventControllerKey::new();
    controller.connect_key_pressed(move |_, key, _, _| {
        if key == gdk::Key::Escape {
            on_escape();
            glib::Propagation::Stop
        } else {
            glib::Propagation::Proceed
        }
    });
    window.add_controller(controller);
}
