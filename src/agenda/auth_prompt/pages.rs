use super::{form, widgets};
use crate::agenda::{AgendaApp, AgendaMsg};
use crate::i18n::translate;
use crate::storage::paths;
use crate::storage::settings::Language;
use crate::ui::{classed_button, label};
use adw::prelude::*;
use relm4::ComponentSender;

pub(super) fn build(
    secret_present: bool,
    token_present: bool,
    authenticating: bool,
    lang: Language,
    sender: ComponentSender<AgendaApp>,
) -> gtk::Box {
    if secret_present {
        saved_client(token_present, authenticating, lang, sender)
    } else {
        missing_client(authenticating, lang, sender)
    }
}

fn missing_client(
    authenticating: bool,
    lang: Language,
    sender: ComponentSender<AgendaApp>,
) -> gtk::Box {
    let (page, body) = shell(
        translate(lang, "create_google_oauth_client"),
        translate(lang, "open_setup_guide_detail"),
    );
    append_setup_actions(&body, lang, sender.clone());
    body.append(&form::credentials(
        authenticating,
        translate(lang, "save_authenticate"),
        lang,
        sender,
    ));
    page
}

fn saved_client(
    token_present: bool,
    authenticating: bool,
    lang: Language,
    sender: ComponentSender<AgendaApp>,
) -> gtk::Box {
    let (page, body) = shell(
        translate(lang, "authorize_calendar_access"),
        translate(lang, "oauth_client_saved_browser"),
    );

    body.append(&widgets::path_summary(
        translate(lang, "oauth_client_saved_at"),
        &paths::client_secret_file(),
    ));
    let replace = gtk::Expander::new(Some(translate(lang, "replace_oauth_client")));
    replace.add_css_class("auth-expander");
    replace.set_child(Some(&form::credentials(
        authenticating,
        translate(lang, "replace_authenticate"),
        lang,
        sender.clone(),
    )));
    body.append(&replace);

    if token_present {
        body.append(&widgets::path_summary(
            translate(lang, "browser_token_saved_at"),
            &paths::oauth_token_file(),
        ));
    }

    append_start_auth_action(&body, token_present, authenticating, lang, sender);
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

fn append_setup_actions(body: &gtk::Box, lang: Language, sender: ComponentSender<AgendaApp>) {
    let actions = widgets::action_row();
    append_setup_guide_button(&actions, lang, sender.clone());
    let open_cloud = classed_button(translate(lang, "google_cloud"), &["action-button"]);
    {
        let sender = sender.clone();
        open_cloud.connect_clicked(move |_| sender.input(AgendaMsg::OpenGoogleCloud));
    }
    actions.append(&open_cloud);

    let open_api = classed_button(translate(lang, "calendar_api"), &["action-button"]);
    {
        let sender = sender.clone();
        open_api.connect_clicked(move |_| sender.input(AgendaMsg::OpenCalendarApi));
    }
    actions.append(&open_api);

    body.append(&actions);
}

fn append_setup_guide_button(
    actions: &gtk::Box,
    lang: Language,
    sender: ComponentSender<AgendaApp>,
) {
    let setup_guide = classed_button(translate(lang, "setup_guide"), &["action-button"]);
    setup_guide.add_css_class("primary-action");
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
    lang: Language,
    sender: ComponentSender<AgendaApp>,
) {
    let actions = widgets::action_row();
    let start_auth = classed_button(
        if authenticating {
            translate(lang, "authenticating")
        } else if token_present {
            translate(lang, "reauthenticate")
        } else {
            translate(lang, "start_authentication")
        },
        &["action-button"],
    );
    start_auth.add_css_class("primary-action");
    start_auth.set_sensitive(!authenticating);
    {
        let sender = sender.clone();
        start_auth.connect_clicked(move |_| sender.input(AgendaMsg::StartAuth));
    }
    actions.append(&start_auth);
    body.append(&actions);
}
