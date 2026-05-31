use super::{
    appearance::AppearanceWidgets,
    calendar::{CalendarWidgets, selected_week_start_from_dropdown},
    language,
};
use crate::agenda::{AgendaApp, AgendaMsg, SettingsChanges};
use crate::i18n::translate;
use crate::storage::settings::Language;
use crate::ui::{classed_button, label};
use adw::prelude::*;
use relm4::ComponentSender;

pub(super) struct FooterWidgets {
    pub(super) container: gtk::Box,
    message_label: gtk::Label,
    cancel_button: gtk::Button,
    apply_button: gtk::Button,
    save_button: gtk::Button,
}

pub(super) fn build(
    lang: Language,
    calendar: &CalendarWidgets,
    appearance: &AppearanceWidgets,
    sender: ComponentSender<AgendaApp>,
) -> FooterWidgets {
    let container = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    container.add_css_class("settings-footer");
    let message_label = label("", &["muted", "settings-message"], 0.0, true);
    message_label.set_hexpand(true);
    container.append(&message_label);

    let spacer = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    spacer.set_hexpand(true);
    container.append(&spacer);

    let cancel_button = classed_button(translate(lang, "cancel"), &["action-button"]);
    {
        let sender = sender.clone();
        cancel_button.connect_clicked(move |_| sender.input(AgendaMsg::CloseSettings));
    }
    container.append(&cancel_button);

    let apply_button = classed_button(translate(lang, "apply"), &["action-button"]);
    {
        let sender = sender.clone();
        let calendar_entry = calendar.calendar_entry.clone();
        let timezone_entry = calendar.timezone_entry.clone();
        let theme_entry = appearance.theme_entry.clone();
        let language_dropdown = appearance.language_dropdown.clone();
        let week_start_dropdown = calendar.week_start_dropdown.clone();
        apply_button.connect_clicked(move |_| {
            sender.input(AgendaMsg::ApplySettings(SettingsChanges {
                calendar: calendar_entry.text().to_string(),
                timezone: timezone_entry.text().to_string(),
                theme_path: theme_entry.text().to_string(),
                language: language::selected(&language_dropdown),
                week_start: selected_week_start_from_dropdown(&week_start_dropdown),
            }));
        });
    }
    container.append(&apply_button);

    let save_button = classed_button(translate(lang, "save"), &["action-button"]);
    save_button.add_css_class("primary-action");
    {
        let sender = sender.clone();
        let calendar_entry = calendar.calendar_entry.clone();
        let timezone_entry = calendar.timezone_entry.clone();
        let theme_entry = appearance.theme_entry.clone();
        let language_dropdown = appearance.language_dropdown.clone();
        let week_start_dropdown = calendar.week_start_dropdown.clone();
        save_button.connect_clicked(move |_| {
            sender.input(AgendaMsg::SaveSettings(SettingsChanges {
                calendar: calendar_entry.text().to_string(),
                timezone: timezone_entry.text().to_string(),
                theme_path: theme_entry.text().to_string(),
                language: language::selected(&language_dropdown),
                week_start: selected_week_start_from_dropdown(&week_start_dropdown),
            }));
        });
    }
    container.append(&save_button);

    FooterWidgets {
        container,
        message_label,
        cancel_button,
        apply_button,
        save_button,
    }
}

pub(super) fn update_text(widgets: &FooterWidgets, lang: Language) {
    widgets.cancel_button.set_label(translate(lang, "cancel"));
    widgets.apply_button.set_label(translate(lang, "apply"));
    widgets.save_button.set_label(translate(lang, "save"));
}

pub(super) fn update_message(widgets: &FooterWidgets, lang: Language, message: Option<&str>) {
    if let Some(message) = message {
        widgets.message_label.set_text(match message {
            "Logged out successfully." => translate(lang, "logged_out_success"),
            "Logged out. Please authenticate." => translate(lang, "logged_out_please_auth"),
            _ => message,
        });
    } else {
        widgets.message_label.set_text("");
    }
}
