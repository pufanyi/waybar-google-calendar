use super::{form, widgets};
use crate::agenda::{AgendaApp, AgendaMsg};
use crate::storage::paths;
use crate::ui::{classed_button, label};
use adw::prelude::*;
use relm4::ComponentSender;

pub(super) fn build(
    page: usize,
    secret_present: bool,
    token_present: bool,
    authenticating: bool,
    sender: ComponentSender<AgendaApp>,
) -> gtk::Box {
    match page {
        0 => intro(sender),
        1 => open_cloud(sender),
        2 => enable_api(sender),
        3 => consent_screen(sender),
        4 => create_client(sender),
        _ => finish(secret_present, token_present, authenticating, sender),
    }
}

fn intro(sender: ComponentSender<AgendaApp>) -> gtk::Box {
    let (page, body) = shell(
        "Use the setup guide",
        "The detailed Google Cloud steps live in docs/google-oauth.md.",
    );
    body.append(&instruction_list(&[
        "Open the setup guide and follow it in your browser.",
        "Create a Desktop app OAuth client in Google Cloud.",
        "Copy the Client ID and Client Secret back into this app.",
    ]));
    append_guide_action(&body, sender);
    page
}

fn open_cloud(sender: ComponentSender<AgendaApp>) -> gtk::Box {
    let (page, body) = shell(
        "Open Google Cloud",
        "First open the Google Auth Platform page and choose a project for this local setup.",
    );
    body.append(&instruction_list(&[
        "Open Google Cloud from here or from the setup guide.",
        "Choose or create the project described in docs/google-oauth.md.",
        "Return here after the project is ready.",
    ]));
    append_guide_and_cloud_actions(&body, sender);
    page
}

fn enable_api(sender: ComponentSender<AgendaApp>) -> gtk::Box {
    let (page, body) = shell(
        "Enable Calendar API",
        "Google will not allow calendar access until the Calendar API is enabled for the project.",
    );
    body.append(&instruction_list(&[
        "Open the Calendar API page.",
        "Check that Google Cloud is using the project from the setup guide.",
        "Click Enable, or continue if the page already says Manage.",
    ]));
    let actions = widgets::action_row();
    append_setup_guide_button(&actions, sender.clone());
    let open_api = classed_button("Open Calendar API", &["action-button"]);
    {
        let sender = sender.clone();
        open_api.connect_clicked(move |_| sender.input(AgendaMsg::OpenCalendarApi));
    }
    actions.append(&open_api);
    body.append(&actions);
    page
}

fn consent_screen(sender: ComponentSender<AgendaApp>) -> gtk::Box {
    let (page, body) = shell(
        "Set up app details",
        "Google uses these details on the permission screen that appears when you sign in.",
    );
    body.append(&instruction_list(&[
        "Use the app details from docs/google-oauth.md.",
        "For personal Gmail, choose External if Google asks for an audience.",
        "Add your own Google account as a test user if Google asks.",
    ]));
    append_guide_and_cloud_actions(&body, sender);
    page
}

fn create_client(sender: ComponentSender<AgendaApp>) -> gtk::Box {
    let (page, body) = shell(
        "Create a Desktop client",
        "This creates the Client ID and Client Secret that this app needs.",
    );
    body.append(&instruction_list(&[
        "Open the Clients page from here or from the setup guide.",
        "Create an OAuth client with Application type Desktop app.",
        "Copy both the Client ID and Client Secret.",
    ]));
    let note = label(
        "If Google hides the secret later, create a new Desktop app client.",
        &["muted", "auth-note"],
        0.0,
        true,
    );
    body.append(&note);
    append_guide_and_cloud_actions(&body, sender);
    page
}

fn finish(
    secret_present: bool,
    token_present: bool,
    authenticating: bool,
    sender: ComponentSender<AgendaApp>,
) -> gtk::Box {
    let (page, body) = shell(
        "Paste and authorize",
        "Finish by saving the OAuth client, then approve read-only Calendar access in your browser.",
    );

    if secret_present {
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
    } else {
        body.append(&instruction_list(&[
            "Follow docs/google-oauth.md to create a Desktop app OAuth client.",
            "Paste the Client ID exactly as Google shows it.",
            "Paste the Client Secret exactly as Google shows it.",
            "Click Save & Authenticate. Your browser will open the Google permission screen.",
        ]));
        append_guide_action(&body, sender.clone());
        body.append(&form::credentials(
            authenticating,
            "Save & Authenticate",
            sender.clone(),
        ));
    }

    if token_present {
        body.append(&widgets::path_summary(
            "Browser token saved at",
            &paths::oauth_token_file(),
        ));
    }

    if secret_present {
        append_start_auth_action(&body, token_present, authenticating, sender);
    }

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

fn instruction_list(items: &[&str]) -> gtk::Box {
    let list = gtk::Box::new(gtk::Orientation::Vertical, 7);
    list.add_css_class("auth-instruction-list");
    for (index, item) in items.iter().enumerate() {
        let row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        row.add_css_class("auth-instruction-row");
        row.append(&label(
            &(index + 1).to_string(),
            &["auth-instruction-index"],
            0.5,
            false,
        ));
        let text = label(item, &["auth-instruction-text", "muted"], 0.0, true);
        text.set_hexpand(true);
        row.append(&text);
        list.append(&row);
    }
    list
}

fn append_guide_action(body: &gtk::Box, sender: ComponentSender<AgendaApp>) {
    let actions = widgets::action_row();
    append_setup_guide_button(&actions, sender);
    body.append(&actions);
}

fn append_guide_and_cloud_actions(body: &gtk::Box, sender: ComponentSender<AgendaApp>) {
    let actions = widgets::action_row();
    append_setup_guide_button(&actions, sender.clone());
    let open_cloud = classed_button("Google Cloud", &["action-button"]);
    {
        let sender = sender.clone();
        open_cloud.connect_clicked(move |_| sender.input(AgendaMsg::OpenGoogleCloud));
    }
    actions.append(&open_cloud);
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
