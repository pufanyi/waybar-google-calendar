use crate::ui::label;
use adw::prelude::*;

#[derive(Clone, Copy)]
pub(super) enum SettingsIcon {
    Calendar,
    Sparkle,
    Account,
}

impl SettingsIcon {
    fn icon_name(self) -> &'static str {
        match self {
            Self::Calendar => "x-office-calendar-symbolic",
            Self::Sparkle => "preferences-desktop-theme-symbolic",
            Self::Account => "avatar-default-symbolic",
        }
    }
}

pub(super) fn section(title: &gtk::Label, icon: SettingsIcon, tint: &str) -> gtk::Box {
    let section = gtk::Box::new(gtk::Orientation::Vertical, 10);
    section.add_css_class("settings-section");

    let header = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    header.add_css_class("settings-section-header");
    header.append(&icon_tile(icon, tint));
    header.append(title);
    section.append(&header);

    section
}

pub(super) fn field_row(label: &gtk::Label, input: &impl IsA<gtk::Widget>) -> gtk::Box {
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    row.add_css_class("settings-form-row");
    label.set_size_request(150, -1);
    label.set_valign(gtk::Align::Center);
    row.append(label);
    input.as_ref().add_css_class("text-entry");
    input.as_ref().set_hexpand(true);
    input.as_ref().set_valign(gtk::Align::Center);
    row.append(input);
    row
}

fn icon_tile(icon: SettingsIcon, tint: &str) -> gtk::Box {
    let tile = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    tile.add_css_class("settings-icon-tile");
    tile.add_css_class(tint);
    tile.set_halign(gtk::Align::Center);
    tile.set_valign(gtk::Align::Center);
    tile.set_size_request(30, 30);

    let glyph = gtk::Image::from_icon_name(icon.icon_name());
    glyph.add_css_class("settings-icon-glyph");
    glyph.set_pixel_size(18);
    glyph.set_halign(gtk::Align::Center);
    glyph.set_valign(gtk::Align::Center);
    glyph.set_margin_top(6);
    glyph.set_margin_bottom(6);
    glyph.set_margin_start(6);
    glyph.set_margin_end(6);
    tile.append(&glyph);

    tile
}

pub(super) fn section_title(text: &str) -> gtk::Label {
    label(text, &["event-title"], 0.0, false)
}
