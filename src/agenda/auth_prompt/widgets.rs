use crate::agenda::{AgendaApp, AgendaMsg};
use crate::i18n::translate;
use crate::storage::settings::Language;
use crate::ui::{classed_button, label};
use adw::prelude::*;
use relm4::ComponentSender;
use std::path::Path;

pub(super) fn utility_actions(
    authenticating: bool,
    lang: Language,
    sender: ComponentSender<AgendaApp>,
) -> gtk::Expander {
    let expander = gtk::Expander::new(Some(translate(lang, "advanced")));
    expander.add_css_class("auth-expander");

    let actions = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    actions.add_css_class("auth-helper-actions");

    let refresh = classed_button(translate(lang, "refresh_status"), &["action-button"]);
    let open_config = classed_button(translate(lang, "config_folder"), &["action-button"]);
    let open_token = classed_button(translate(lang, "token_folder"), &["action-button"]);

    refresh.set_sensitive(!authenticating);

    {
        let sender = sender.clone();
        refresh.connect_clicked(move |_| sender.input(AgendaMsg::Refresh));
    }
    {
        let sender = sender.clone();
        open_config.connect_clicked(move |_| sender.input(AgendaMsg::OpenConfigDir));
    }
    {
        let sender = sender.clone();
        open_token.connect_clicked(move |_| sender.input(AgendaMsg::OpenTokenDir));
    }

    actions.append(&refresh);
    actions.append(&open_config);
    actions.append(&open_token);
    expander.set_child(Some(&actions));
    expander
}

pub(super) fn path_summary(title: &str, path: &Path) -> gtk::Box {
    let row = gtk::Box::new(gtk::Orientation::Vertical, 3);
    row.add_css_class("auth-path-row");
    row.append(&label(title, &["field-label"], 0.0, false));
    let path_label = label(
        &path.display().to_string(),
        &["path-label", "muted"],
        0.0,
        false,
    );
    path_label.set_selectable(true);
    row.append(&path_label);
    row
}

pub(super) fn action_row() -> gtk::Box {
    let actions = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    actions.add_css_class("auth-step-actions");
    actions
}
