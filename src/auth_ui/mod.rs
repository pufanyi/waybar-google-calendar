use crate::google;
use crate::storage::paths;
use crate::ui::{add_escape_action, classed_button, label};
use adw::prelude::*;
use gtk::gio;
use relm4::{Component, ComponentParts, ComponentSender};
use std::fs;
use std::path::{Path, PathBuf};

const GOOGLE_CLOUD_CREDENTIALS_URL: &str = "https://console.cloud.google.com/apis/credentials";

#[derive(Debug, Clone)]
pub struct AuthApp {
    status: AuthStatus,
    message: String,
    loading: bool,
}

#[derive(Debug, Clone)]
struct AuthStatus {
    secret_path: PathBuf,
    token_path: PathBuf,
    secret_present: bool,
    token_present: bool,
}

#[derive(Debug)]
pub enum AuthMsg {
    Start,
    Refresh,
    OpenConfigDir,
    OpenTokenDir,
    OpenGoogleCloud,
    Close,
}

#[derive(Debug)]
pub enum AuthCommandOutput {
    Finished(Result<(), String>),
}

pub struct AuthWidgets {
    secret_path: gtk::Label,
    secret_badge: gtk::Label,
    token_path: gtk::Label,
    token_badge: gtk::Label,
    message: gtk::Label,
    start: gtk::Button,
    refresh: gtk::Button,
}

impl Component for AuthApp {
    type Init = ();
    type Input = AuthMsg;
    type Output = ();
    type CommandOutput = AuthCommandOutput;
    type Root = adw::ApplicationWindow;
    type Widgets = AuthWidgets;

    fn init_root() -> Self::Root {
        let root = adw::ApplicationWindow::builder()
            .title("Google Calendar Auth")
            .default_width(640)
            .default_height(380)
            .resizable(false)
            .build();
        root.set_decorated(false);
        root
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let root_box = gtk::Box::new(gtk::Orientation::Vertical, 12);
        root_box.add_css_class("panel");

        let topbar = gtk::Box::new(gtk::Orientation::Horizontal, 10);
        topbar.add_css_class("topbar");
        topbar.append(&label("Google Calendar Auth", &["title"], 0.0, false));

        let spacer = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        spacer.set_hexpand(true);
        topbar.append(&spacer);

        let close = classed_button("x", &["close-button"]);
        {
            let sender = sender.clone();
            close.connect_clicked(move |_| sender.input(AuthMsg::Close));
        }
        topbar.append(&close);
        root_box.append(&topbar);

        let card = gtk::Box::new(gtk::Orientation::Vertical, 10);
        card.add_css_class("settings-card");

        let secret_path = path_label("");
        let secret_badge = badge_label("");
        card.append(&settings_row(
            "OAuth client secret",
            &secret_path,
            &secret_badge,
        ));

        let token_path = path_label("");
        let token_badge = badge_label("");
        card.append(&settings_row("Token cache", &token_path, &token_badge));
        root_box.append(&card);

        let actions = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        let start = classed_button("Start Authentication", &["action-button"]);
        let refresh = classed_button("Refresh", &["action-button"]);
        let open_config = classed_button("Open Config Folder", &["action-button"]);
        let open_token = classed_button("Open Token Folder", &["action-button"]);
        let open_cloud = classed_button("Google Cloud", &["action-button"]);

        {
            let sender = sender.clone();
            start.connect_clicked(move |_| sender.input(AuthMsg::Start));
        }
        {
            let sender = sender.clone();
            refresh.connect_clicked(move |_| sender.input(AuthMsg::Refresh));
        }
        {
            let sender = sender.clone();
            open_config.connect_clicked(move |_| sender.input(AuthMsg::OpenConfigDir));
        }
        {
            let sender = sender.clone();
            open_token.connect_clicked(move |_| sender.input(AuthMsg::OpenTokenDir));
        }
        {
            let sender = sender.clone();
            open_cloud.connect_clicked(move |_| sender.input(AuthMsg::OpenGoogleCloud));
        }

        actions.append(&start);
        actions.append(&refresh);
        actions.append(&open_config);
        actions.append(&open_token);
        actions.append(&open_cloud);
        root_box.append(&actions);

        let message = label("", &["muted", "auth-message"], 0.0, true);
        root_box.append(&message);

        root.set_content(Some(&root_box));
        {
            let sender = sender.clone();
            add_escape_action(&root, move || sender.input(AuthMsg::Close));
        }

        let model = AuthApp {
            status: AuthStatus::read(),
            message: "Ready".to_string(),
            loading: false,
        };
        let mut widgets = AuthWidgets {
            secret_path,
            secret_badge,
            token_path,
            token_badge,
            message,
            start,
            refresh,
        };
        render_auth(&model, &mut widgets);

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>, root: &Self::Root) {
        match message {
            AuthMsg::Start => {
                if self.loading {
                    return;
                }
                self.loading = true;
                self.message = "Opening browser for Google OAuth...".to_string();
                sender
                    .spawn_oneshot_command(|| AuthCommandOutput::Finished(google::auth_calendar()));
            }
            AuthMsg::Refresh => {
                self.status = AuthStatus::read();
                self.message = status_message(&self.status);
            }
            AuthMsg::OpenConfigDir => {
                self.message = open_dir(&paths::config_dir())
                    .map(|_| "Config folder opened.".to_string())
                    .unwrap_or_else(|error| error);
                self.status = AuthStatus::read();
            }
            AuthMsg::OpenTokenDir => {
                self.message = open_dir(&paths::data_dir())
                    .map(|_| "Token folder opened.".to_string())
                    .unwrap_or_else(|error| error);
                self.status = AuthStatus::read();
            }
            AuthMsg::OpenGoogleCloud => {
                self.message = google::open_external_uri(GOOGLE_CLOUD_CREDENTIALS_URL)
                    .map(|_| "Google Cloud opened in your browser.".to_string())
                    .unwrap_or_else(|error| error);
            }
            AuthMsg::Close => {
                root.close();
            }
        }
    }

    fn update_cmd(
        &mut self,
        output: Self::CommandOutput,
        _sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        self.loading = false;
        self.status = AuthStatus::read();
        match output {
            AuthCommandOutput::Finished(Ok(())) => {
                self.message = "Google Calendar authenticated.".to_string();
            }
            AuthCommandOutput::Finished(Err(error)) => {
                self.message = error;
            }
        }
    }

    fn update_view(&self, widgets: &mut Self::Widgets, _sender: ComponentSender<Self>) {
        render_auth(self, widgets);
    }
}

impl AuthStatus {
    fn read() -> Self {
        let secret_path = paths::client_secret_file();
        let token_path = paths::oauth_token_file();
        Self {
            secret_present: secret_path.exists(),
            token_present: token_path.exists(),
            secret_path,
            token_path,
        }
    }
}

fn render_auth(model: &AuthApp, widgets: &mut AuthWidgets) {
    widgets
        .secret_path
        .set_text(&model.status.secret_path.display().to_string());
    widgets
        .token_path
        .set_text(&model.status.token_path.display().to_string());
    set_badge(&widgets.secret_badge, model.status.secret_present);
    set_badge(&widgets.token_badge, model.status.token_present);
    widgets.message.set_text(&model.message);
    widgets
        .start
        .set_sensitive(model.status.secret_present && !model.loading);
    widgets.refresh.set_sensitive(!model.loading);
    widgets.start.set_label(if model.loading {
        "Authenticating"
    } else {
        "Start Authentication"
    });
}

fn settings_row(title: &str, path: &gtk::Label, badge: &gtk::Label) -> gtk::Box {
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    row.add_css_class("settings-row");

    let labels = gtk::Box::new(gtk::Orientation::Vertical, 2);
    labels.set_hexpand(true);
    labels.append(&label(title, &["event-title"], 0.0, false));
    labels.append(path);

    row.append(&labels);
    row.append(badge);
    row
}

fn path_label(text: &str) -> gtk::Label {
    let widget = label(text, &["path-label", "muted"], 0.0, false);
    widget.set_selectable(true);
    widget
}

fn badge_label(text: &str) -> gtk::Label {
    label(text, &["status-badge"], 0.5, false)
}

fn set_badge(widget: &gtk::Label, present: bool) {
    widget.remove_css_class("success");
    widget.remove_css_class("warning");
    if present {
        widget.set_text("Ready");
        widget.add_css_class("success");
    } else {
        widget.set_text("Missing");
        widget.add_css_class("warning");
    }
}

fn status_message(status: &AuthStatus) -> String {
    match (status.secret_present, status.token_present) {
        (false, _) => "Missing OAuth client secret.".to_string(),
        (true, false) => "Ready to authenticate.".to_string(),
        (true, true) => "Authenticated.".to_string(),
    }
}

fn open_dir(path: &Path) -> Result<(), String> {
    fs::create_dir_all(path)
        .map_err(|err| format!("Could not create folder {}: {err}", path.display()))?;
    let file = gio::File::for_path(path);
    let uri = file.uri();
    google::open_external_uri(uri.as_str())
}
