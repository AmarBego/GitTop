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

    // Sync all proxy settings from temp fields
    settings.proxy.enabled = state.enabled;
    settings.proxy.url = new_url.clone();

    // Update has_credentials flag
    settings.proxy.has_credentials = !state.username.is_empty() || !state.password.is_empty();

    // Case 1: URL changed - handle both old and new URLs
    if old_url != new_url {
        eprintln!(
            "[PROXY] URL changed: '{}' -> '{}' (enabled: {})",
            old_url, new_url, state.enabled
        );

        // Delete credentials for old URL to prevent orphaned data
        if !old_url.is_empty() {
            eprintln!("[PROXY] Deleting credentials for old proxy URL");
            let _ = proxy_keyring::delete_proxy_credentials(&old_url);
        }

        // Save credentials for new URL if provided
        if !state.username.is_empty() && !state.password.is_empty() {
            eprintln!("[PROXY] Saving credentials for new proxy URL");
            let username = state.username.as_str();
            let password = state.password.as_str();
            let _ = proxy_keyring::save_proxy_credentials(&new_url, username, password);
        }
    }
    // Case 2: URL unchanged - only handle credential changes
    else if state.username.is_empty() && state.password.is_empty() {
        eprintln!("[PROXY] Credentials cleared, deleting from keyring");
        let _ = proxy_keyring::delete_proxy_credentials(&old_url);
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
            eprintln!("[PROXY] Credentials changed, saving to keyring");
            let username = state.username.as_str();
            let password = state.password.as_str();
            let _ = proxy_keyring::save_proxy_credentials(&new_url, username, password);
        } else {
            eprintln!("[PROXY] Credentials unchanged, skipping keyring write");
        }
    }

    eprintln!(
        "[PROXY] Settings saved: enabled={}, url='{}', has_credentials={}",
        settings.proxy.enabled, settings.proxy.url, settings.proxy.has_credentials
    );

    // Signal that clients need rebuild when leaving settings
    state.needs_rebuild = true;
    // Reset dirty flag since we just saved
    state.creds_dirty = false;

    // Persist settings
    let _ = settings.save();
    crate::platform::trim_memory();
}
