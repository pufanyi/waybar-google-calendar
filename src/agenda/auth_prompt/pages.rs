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
        0 => intro(),
        1 => open_cloud(sender),
        2 => enable_api(sender),
        3 => consent_screen(sender),
        4 => create_client(sender),
        _ => finish(secret_present, token_present, authenticating, sender),
    }
}

fn intro() -> gtk::Box {
    let (page, body) = shell(
        "Before you start",
        "You will create a private Google OAuth client, then let this app read your calendar.",
    );
    body.append(&instruction_list(&[
        "Keep this calendar window open while the browser opens Google Cloud.",
        "Use the Google account whose calendar you want to show in Waybar.",
        "This app asks for read-only Calendar access and saves the credentials on this computer.",
        "If a Google page looks different, choose the closest option with the same name.",
    ]));
    page
}

fn open_cloud(sender: ComponentSender<AgendaApp>) -> gtk::Box {
    let (page, body) = shell(
        "Open Google Cloud",
        "First open the Google Auth Platform page and choose a project for this local setup.",
    );
    body.append(&instruction_list(&[
        "Click Google Cloud below. Your browser should open a Google Cloud page.",
        "Sign in if Google asks.",
        "If Google asks for a project, create one named Waybar Calendar or choose an existing personal project.",
        "If you see a Get started button for Google Auth Platform, click it.",
    ]));
    append_cloud_action(&body, sender);
    page
}

fn enable_api(sender: ComponentSender<AgendaApp>) -> gtk::Box {
    let (page, body) = shell(
        "Enable Calendar API",
        "Google will not allow calendar access until the Calendar API is enabled for the project.",
    );
    body.append(&instruction_list(&[
        "Click Open Calendar API below.",
        "Check the project selector near the top of the page. It should be the project you chose for this app.",
        "Click Enable. If the button says Manage, the API is already enabled.",
        "When it finishes, return here and click Next.",
    ]));
    let actions = widgets::action_row();
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
        "If Google asks for app information, use Waybar Google Calendar as the app name.",
        "For User support email and Developer contact email, choose your own Google email.",
        "Choose External for a personal Gmail account. Choose Internal only for a Google Workspace organization.",
        "If there is a Test users page, add the same Google email you will use for Calendar.",
        "Save or continue until Google lets you create OAuth clients.",
    ]));
    append_cloud_action(&body, sender);
    page
}

fn create_client(sender: ComponentSender<AgendaApp>) -> gtk::Box {
    let (page, body) = shell(
        "Create a Desktop client",
        "This creates the Client ID and Client Secret that this app needs.",
    );
    body.append(&instruction_list(&[
        "Open the Google Auth Platform Clients page.",
        "Click Create client.",
        "For Application type, choose Desktop app.",
        "For Name, enter Waybar Google Calendar, then click Create.",
        "Copy both the Client ID and Client Secret before closing the result window.",
    ]));
    let note = label(
        "Google may only show the full client secret when it is created. Copy it immediately.",
        &["muted", "auth-note"],
        0.0,
        true,
    );
    body.append(&note);
    append_cloud_action(&body, sender);
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
            "Paste the Client ID exactly as Google shows it.",
            "Paste the Client Secret exactly as Google shows it.",
            "Click Save & Authenticate. Your browser will open the Google permission screen.",
        ]));
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

fn append_cloud_action(body: &gtk::Box, sender: ComponentSender<AgendaApp>) {
    let actions = widgets::action_row();
    let open_cloud = classed_button("Google Cloud", &["action-button"]);
    {
        let sender = sender.clone();
        open_cloud.connect_clicked(move |_| sender.input(AgendaMsg::OpenGoogleCloud));
    }
    actions.append(&open_cloud);
    body.append(&actions);
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
