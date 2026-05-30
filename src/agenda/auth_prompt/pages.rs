use super::{form, widgets};
use crate::agenda::{AgendaApp, AgendaMsg};
use crate::storage::paths;
use crate::ui::{classed_button, label};
use adw::prelude::*;
use relm4::ComponentSender;

pub(super) fn build(
    secret_present: bool,
    token_present: bool,
    authenticating: bool,
    sender: ComponentSender<AgendaApp>,
) -> gtk::Box {
    if secret_present {
        saved_client(token_present, authenticating, sender)
    } else {
        missing_client(authenticating, sender)
    }
}

fn missing_client(authenticating: bool, sender: ComponentSender<AgendaApp>) -> gtk::Box {
    let (page, body) = shell(
        "Create a Google OAuth client",
        "Open the setup guide, create a Desktop app OAuth client, then paste the values here.",
    );
    let note = label(
        "Guide: https://github.com/pufanyi/waybar-google-calendar/blob/main/docs/google-oauth.md",
        &["muted", "auth-note"],
        0.0,
        true,
    );
    body.append(&note);
    append_setup_actions(&body, sender.clone());
    body.append(&form::credentials(
        authenticating,
        "Save & Authenticate",
        sender,
    ));
    page
}

fn saved_client(
    token_present: bool,
    authenticating: bool,
    sender: ComponentSender<AgendaApp>,
) -> gtk::Box {
    let (page, body) = shell(
        "Authorize calendar access",
        "The OAuth client is saved. Start browser authorization to load your calendar.",
    );

    body.append(&widgets::path_summary(
        "OAuth client saved at",
        &paths::client_secret_file(),
    ));
    let replace = gtk::Expander::new(Some("Replace OAuth client"));
    replace.add_css_class("auth-expander");
    replace.set_child(Some(&form::credentials(
        authenticating,
        "Replace & Authenticate",
        sender.clone(),
    )));
    body.append(&replace);

    if token_present {
        body.append(&widgets::path_summary(
            "Browser token saved at",
            &paths::oauth_token_file(),
        ));
    }

    append_start_auth_action(&body, token_present, authenticating, sender);
    page
}

fn shell(title: &str, detail: &str) -> (gtk::Box, gtk::Box) {
    let page = gtk::Box::new(gtk::Orientation::Vertical, 10);
    page.add_css_class("auth-wizard-page");
    page.append(&label(
        title,
        &["event-title", "auth-wizard-title"],
        0.0,
        false,
    ));
    page.append(&label(detail, &["muted", "auth-step-detail"], 0.0, true));

    let body = gtk::Box::new(gtk::Orientation::Vertical, 9);
    body.add_css_class("auth-wizard-body");
    page.append(&body);
    (page, body)
}

fn append_setup_actions(body: &gtk::Box, sender: ComponentSender<AgendaApp>) {
    let actions = widgets::action_row();
    append_setup_guide_button(&actions, sender.clone());
    let open_cloud = classed_button("Google Cloud", &["action-button"]);
    {
        let sender = sender.clone();
        open_cloud.connect_clicked(move |_| sender.input(AgendaMsg::OpenGoogleCloud));
    }
    actions.append(&open_cloud);

    let open_api = classed_button("Calendar API", &["action-button"]);
    {
        let sender = sender.clone();
        open_api.connect_clicked(move |_| sender.input(AgendaMsg::OpenCalendarApi));
    }
    actions.append(&open_api);

    body.append(&actions);
}

fn append_setup_guide_button(actions: &gtk::Box, sender: ComponentSender<AgendaApp>) {
    let setup_guide = classed_button("Setup Guide", &["action-button"]);
    {
        let sender = sender.clone();
        setup_guide.connect_clicked(move |_| sender.input(AgendaMsg::OpenSetupGuide));
    }
    actions.append(&setup_guide);
}

fn append_start_auth_action(
    body: &gtk::Box,
    token_present: bool,
    authenticating: bool,
    sender: ComponentSender<AgendaApp>,
) {
    let actions = widgets::action_row();
    let start_auth = classed_button(
        if authenticating {
            "Authenticating"
        } else if token_present {
            "Re-authenticate"
        } else {
            "Start Authentication"
        },
        &["action-button"],
    );
    start_auth.set_sensitive(!authenticating);
    {
        let sender = sender.clone();
        start_auth.connect_clicked(move |_| sender.input(AgendaMsg::StartAuth));
    }
    actions.append(&start_auth);
    body.append(&actions);
}
