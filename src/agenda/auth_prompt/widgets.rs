use crate::agenda::{AgendaApp, AgendaMsg};
use crate::ui::{classed_button, label};
use adw::prelude::*;
use relm4::ComponentSender;
use std::path::Path;

pub(super) fn wizard_progress(current_page: usize, page_count: usize) -> gtk::Box {
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 7);
    row.add_css_class("auth-wizard-progress");
    for index in 0..page_count {
        let dot = label(&(index + 1).to_string(), &["auth-progress-dot"], 0.5, false);
        if index == current_page {
            dot.add_css_class("active");
        } else if index < current_page {
            dot.add_css_class("completed");
        }
        row.append(&dot);
    }
    row
}

pub(super) fn wizard_navigation(
    page: usize,
    page_count: usize,
    authenticating: bool,
    sender: ComponentSender<AgendaApp>,
) -> gtk::Box {
    let last_page = page_count.saturating_sub(1);
    let navigation = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    navigation.add_css_class("auth-wizard-navigation");

    let previous = classed_button("Previous", &["action-button"]);
    previous.set_sensitive(page > 0 && !authenticating);
    let next = classed_button("Next", &["action-button"]);
    next.set_sensitive(page < last_page && !authenticating);
    let page_label = label(
        &format!("Step {} of {}", page + 1, page_count),
        &["muted", "auth-page-label"],
        0.5,
        false,
    );
    page_label.set_hexpand(true);

    {
        let sender = sender.clone();
        previous.connect_clicked(move |_| sender.input(AgendaMsg::PreviousAuthPage));
    }
    {
        let sender = sender.clone();
        next.connect_clicked(move |_| sender.input(AgendaMsg::NextAuthPage));
    }

    navigation.append(&previous);
    navigation.append(&page_label);
    navigation.append(&next);
    navigation
}

pub(super) fn utility_actions(
    authenticating: bool,
    sender: ComponentSender<AgendaApp>,
) -> gtk::Expander {
    let expander = gtk::Expander::new(Some("Advanced"));
    expander.add_css_class("auth-expander");

    let actions = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    actions.add_css_class("auth-helper-actions");

    let refresh = classed_button("Refresh Status", &["action-button"]);
    let open_config = classed_button("Config Folder", &["action-button"]);
    let open_token = classed_button("Token Folder", &["action-button"]);

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
