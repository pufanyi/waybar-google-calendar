use super::transport::open_external_uri;
use super::types::{ClientSecretFile, InstalledClientSecret};
use super::{CALENDAR_SCOPE, fetch_timeout, runtime};
use crate::calendar::model::FETCH_TIMEOUT_SECONDS;
use crate::storage::paths;
use std::fs;
use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;
use std::time::Duration;
use yup_oauth2::authenticator_delegate::InstalledFlowDelegate;
use yup_oauth2::{InstalledFlowAuthenticator, InstalledFlowReturnMethod};

pub fn auth_calendar() -> Result<(), String> {
    runtime()?.block_on(async {
        let timeout = fetch_timeout(FETCH_TIMEOUT_SECONDS);
        let _ = access_token(timeout, false).await?;
        Ok(())
    })
}

pub fn save_client_secret(client_id: &str, client_secret: &str) -> Result<PathBuf, String> {
    let client_id = client_id.trim();
    let client_secret = client_secret.trim();
    if client_id.is_empty() {
        return Err("Client ID is empty.".to_string());
    }
    if client_secret.is_empty() {
        return Err("Client Secret is empty.".to_string());
    }

    let secret_file = paths::client_secret_file();
    if let Some(parent) = secret_file.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| format!("Could not create folder {}: {err}", parent.display()))?;
    }

    let payload = ClientSecretFile {
        installed: InstalledClientSecret {
            client_id,
            project_id: "",
            auth_uri: "https://accounts.google.com/o/oauth2/auth",
            token_uri: "https://oauth2.googleapis.com/token",
            auth_provider_x509_cert_url: "https://www.googleapis.com/oauth2/v1/certs",
            client_secret,
            redirect_uris: &["http://localhost"],
        },
    };
    let json = serde_json::to_string_pretty(&payload)
        .map_err(|err| format!("Could not build Google OAuth client secret JSON: {err}"))?;
    fs::write(&secret_file, json)
        .map_err(|err| format!("Could not write {}: {err}", secret_file.display()))?;
    secure_file(&secret_file);
    Ok(secret_file)
}

pub(super) async fn access_token(
    timeout: u64,
    require_existing_token: bool,
) -> Result<String, String> {
    let secret_file = paths::client_secret_file();
    if !secret_file.exists() {
        return Err(format!(
            "Missing Google OAuth client secret. Paste Client ID and Client Secret in the app, put the JSON at {}, or set WAYBAR_GCAL_CLIENT_SECRET.",
            secret_file.display()
        ));
    }

    let token_file = paths::oauth_token_file();
    if require_existing_token && !token_file.exists() {
        return Err(
            "Google Calendar is not authenticated. Start authentication from the app or run `waybar-gcal auth` first.".to_string(),
        );
    }

    if let Some(parent) = token_file.parent() {
        create_secure_dir(parent)?;
    }

    let secret = yup_oauth2::read_application_secret(&secret_file)
        .await
        .map_err(|err| {
            format!(
                "Could not read Google OAuth client secret {}: {err}",
                secret_file.display()
            )
        })?;

    let auth = InstalledFlowAuthenticator::builder(secret, InstalledFlowReturnMethod::HTTPRedirect)
        .flow_delegate(Box::new(BrowserFlowDelegate))
        .persist_tokens_to_disk(token_file)
        .with_timeout(Duration::from_secs(timeout))
        .build()
        .await
        .map_err(|err| format!("Could not initialize Google OAuth: {err}"))?;

    let token = auth
        .token(&[CALENDAR_SCOPE])
        .await
        .map_err(|err| format!("Could not authenticate Google Calendar: {err}"))?
        .token()
        .map(ToOwned::to_owned)
        .ok_or_else(|| "Google OAuth did not return an access token.".to_string())?;
    secure_token_file();
    Ok(token)
}

struct BrowserFlowDelegate;

impl InstalledFlowDelegate for BrowserFlowDelegate {
    fn present_user_url<'a>(
        &'a self,
        url: &'a str,
        _need_code: bool,
    ) -> Pin<Box<dyn Future<Output = Result<String, String>> + Send + 'a>> {
        Box::pin(async move {
            open_external_uri(url)?;
            Ok(String::new())
        })
    }
}

fn secure_token_file() {
    secure_file(&paths::oauth_token_file());
}

fn secure_file(path: &std::path::Path) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        if let Ok(metadata) = fs::metadata(path) {
            let mut permissions = metadata.permissions();
            permissions.set_mode(0o600);
            let _ = fs::set_permissions(path, permissions);
        }
    }
}

fn create_secure_dir(path: &std::path::Path) -> Result<(), String> {
    fs::create_dir_all(path)
        .map_err(|err| format!("Could not create token directory {}: {err}", path.display()))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let mut permissions = fs::metadata(path)
            .map_err(|err| format!("Could not read token directory {}: {err}", path.display()))?
            .permissions();
        permissions.set_mode(0o700);
        fs::set_permissions(path, permissions).map_err(|err| {
            format!(
                "Could not secure token directory permissions {}: {err}",
                path.display()
            )
        })?;
    }

    Ok(())
}
