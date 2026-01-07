use super::message::ProxyMessage;
use super::state::NetworkProxyState;
use crate::github::proxy_keyring;
use crate::settings::AppSettings;
use iced::Task;

pub fn update(
    state: &mut NetworkProxyState,
    message: ProxyMessage,
    settings: &mut AppSettings,
) -> Task<ProxyMessage> {
    match message {
        ProxyMessage::ToggleEnabled(enabled) => {
            state.enabled = enabled;
            Task::none()
        }
        ProxyMessage::UrlChanged(url) => {
            state.url = url;
            Task::none()
        }
        ProxyMessage::UsernameChanged(username) => {
            state.username = username;
            state.creds_dirty = true;
            Task::none()
        }
        ProxyMessage::PasswordChanged(password) => {
            state.password = password;
            state.creds_dirty = true;
            Task::none()
        }
        ProxyMessage::Save => {
            update_proxy_credentials(state, settings);
            Task::none()
        }
    }
}

fn update_proxy_credentials(state: &mut NetworkProxyState, settings: &mut AppSettings) {
    let old_url = settings.proxy.url.clone();
    let new_url = state.url.clone();
    let url_changed = old_url != new_url;
    let old_url_set = !old_url.is_empty();
    let new_url_set = !new_url.is_empty();

    // Sync all proxy settings from temp fields
    settings.proxy.enabled = state.enabled;
    settings.proxy.url = new_url.clone();

    // Update has_credentials flag
    settings.proxy.has_credentials = !state.username.is_empty() || !state.password.is_empty();

    // Case 1: URL changed - handle both old and new URLs
    if url_changed {
        tracing::info!(
            old_url_set,
            new_url_set,
            enabled = state.enabled,
            "Proxy URL changed"
        );

        // Delete credentials for old URL to prevent orphaned data
        if old_url_set {
            tracing::debug!("Deleting credentials for old proxy URL");
            if let Err(e) = proxy_keyring::delete_proxy_credentials(&old_url) {
                tracing::warn!(error = %e, "Failed to delete proxy credentials");
            }
        }

        // Save credentials for new URL if provided
        if !state.username.is_empty() && !state.password.is_empty() {
            tracing::debug!("Saving credentials for new proxy URL");
            let username = state.username.as_str();
            let password = state.password.as_str();
            if let Err(e) = proxy_keyring::save_proxy_credentials(&new_url, username, password) {
                tracing::warn!(error = %e, "Failed to save proxy credentials");
            }
        }
    }
    // Case 2: URL unchanged - only handle credential changes
    else if state.username.is_empty() && state.password.is_empty() {
        tracing::debug!("Proxy credentials cleared; deleting from keyring");
        if let Err(e) = proxy_keyring::delete_proxy_credentials(&old_url) {
            tracing::warn!(error = %e, "Failed to delete proxy credentials");
        }
    } else {
        // Check if credentials actually changed
        let should_save = if let Ok(Some((saved_username, saved_password))) =
            proxy_keyring::load_proxy_credentials(&old_url)
        {
            saved_username != state.username || saved_password != state.password
        } else {
            // No existing credentials, so save the new ones
            true
        };

        if should_save {
            tracing::debug!("Proxy credentials changed; saving to keyring");
            let username = state.username.as_str();
            let password = state.password.as_str();
            if let Err(e) = proxy_keyring::save_proxy_credentials(&new_url, username, password) {
                tracing::warn!(error = %e, "Failed to save proxy credentials");
            }
        } else {
            tracing::debug!("Proxy credentials unchanged; skipping keyring write");
        }
    }

    tracing::info!(
        enabled = settings.proxy.enabled,
        url_set = new_url_set,
        has_credentials = settings.proxy.has_credentials,
        "Proxy settings saved"
    );

    // Signal that clients need rebuild when leaving settings
    state.needs_rebuild = true;
    // Reset dirty flag since we just saved
    state.creds_dirty = false;

    // Persist settings
    let _ = settings.save();
    crate::platform::trim_memory();
}
