use crate::agenda::{AgendaApp, AgendaMsg};
use crate::i18n::translate;
use crate::storage::settings::Language;
use crate::ui::{classed_button, label};
use adw::prelude::*;
use relm4::ComponentSender;

pub(super) fn credentials(
    authenticating: bool,
    button_label: &str,
    lang: Language,
    sender: ComponentSender<AgendaApp>,
) -> gtk::Box {
    let form = gtk::Box::new(gtk::Orientation::Vertical, 8);
    form.add_css_class("auth-form");

    let client_id = credential_entry(translate(lang, "client_id"));
    let client_secret = credential_entry(translate(lang, "client_secret"));
    client_secret.set_visibility(false);

    form.append(&field_row(translate(lang, "client_id"), &client_id));
    form.append(&field_row(translate(lang, "client_secret"), &client_secret));

    let save = classed_button(button_label, &["action-button"]);
    save.set_sensitive(!authenticating);
    {
        let client_id = client_id.clone();
        let client_secret = client_secret.clone();
        save.connect_clicked(move |_| {
            sender.input(AgendaMsg::SaveAndStartAuth {
                client_id: client_id.text().trim().to_string(),
                client_secret: client_secret.text().trim().to_string(),
            });
        });
    }
    form.append(&save);
    form
}

fn credential_entry(placeholder: &str) -> gtk::Entry {
    let entry = gtk::Entry::new();
    entry.add_css_class("text-entry");
    entry.set_placeholder_text(Some(placeholder));
    entry.set_hexpand(true);
    entry
}

fn field_row(title: &str, entry: &gtk::Entry) -> gtk::Box {
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    let title = label(title, &["field-label"], 0.0, false);
    title.set_size_request(104, -1);
    row.append(&title);
    row.append(entry);
    row
}
