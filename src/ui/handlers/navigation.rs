//! Navigation handlers - screen transitions.

use iced::Task;

use crate::github::SessionManager;
use crate::settings::AppSettings;
use crate::ui::context::AppContext;
use crate::ui::routing::{RuleEngineOrigin, Screen};
use crate::ui::screens::notifications::NotificationsScreen;
use crate::ui::screens::settings::SettingsScreen;
use crate::ui::screens::settings::rule_engine::RuleEngineScreen;
use crate::ui::screens::settings::rule_engine::rules::NotificationRuleSet;

use super::super::app::Message;

// ============================================================================
// Navigation Results
// ============================================================================

/// Result of navigating to notifications screen.
pub struct NotificationsTransition {
    pub screen: Box<NotificationsScreen>,
    pub task: Task<Message>,
    pub updated_settings: AppSettings,
}

/// Result of navigating to settings screen.
pub struct SettingsTransition {
    pub screen: Box<SettingsScreen>,
    pub updated_settings: AppSettings,
}

/// Result of navigating to rule engine screen.
pub struct RuleEngineTransition {
    pub screen: Box<RuleEngineScreen>,
    pub origin: RuleEngineOrigin,
    pub updated_settings: AppSettings,
}

// ============================================================================
// Navigation Functions
// ============================================================================

/// Navigate to the notifications screen.
///
/// Handles proxy settings changes and client rebuilding.
pub fn go_to_notifications(
    current_screen: &Screen,
    ctx: &mut AppContext,
) -> Option<NotificationsTransition> {
    let settings = match current_screen {
        Screen::Settings(s) => s.settings.clone(),
        _ => ctx.settings.clone(),
    };

    // Check if proxy settings changed and rebuild clients if needed
    let proxy_changed = settings.proxy.enabled != ctx.settings.proxy.enabled
        || settings.proxy.url != ctx.settings.proxy.url
        || settings.proxy.has_credentials != ctx.settings.proxy.has_credentials;

    let needs_rebuild = if let Screen::Settings(s) = current_screen {
        s.proxy.needs_rebuild
    } else {
        false
    };

    if proxy_changed || needs_rebuild {
        eprintln!(
            "[PROXY] Rebuilding clients (settings_changed: {}, needs_rebuild: {})",
            proxy_changed, needs_rebuild
        );

        if let Err(e) = ctx.sessions.rebuild_clients_with_proxy(&settings.proxy) {
            eprintln!("[PROXY] Failed to rebuild clients: {}", e);
        }
    }

    let session = ctx.sessions.primary()?;

    let (notif_screen, task) =
        NotificationsScreen::new(session.client.clone(), session.user.clone());

    Some(NotificationsTransition {
        screen: Box::new(notif_screen),
        task: task.map(Message::Notifications),
        updated_settings: settings,
    })
}

/// Navigate to the settings screen.
pub fn go_to_settings(ctx: &AppContext) -> SettingsTransition {
    let settings = ctx.settings.clone();
    let settings_screen = SettingsScreen::new(settings.clone());

    SettingsTransition {
        screen: Box::new(settings_screen),
        updated_settings: settings,
    }
}

/// Navigate to the rule engine screen.
pub fn go_to_rule_engine(
    current_settings: Option<&AppSettings>,
    origin: RuleEngineOrigin,
) -> RuleEngineTransition {
    let settings = current_settings.cloned().unwrap_or_else(AppSettings::load);

    let rules = NotificationRuleSet::load();
    let rule_engine_screen = RuleEngineScreen::new(rules, settings.clone());

    RuleEngineTransition {
        screen: Box::new(rule_engine_screen),
        origin,
        updated_settings: settings,
    }
}

// ============================================================================
// Account Switching
// ============================================================================

/// Switch to a different account.
///
/// Returns the new screen and task if successful.
pub fn switch_account(
    username: &str,
    current_screen: &NotificationsScreen,
    sessions: &mut SessionManager,
    settings: &mut AppSettings,
) -> Option<(Box<NotificationsScreen>, Task<Message>)> {
    // Skip if already on this account
    if sessions.primary().is_some_and(|s| s.username == username) {
        return None;
    }

    // Preserve cross-account priority notifications
    let cross_account_priority = current_screen.get_cross_account_priority();
    sessions.set_primary(username);

    // Persist the active account preference
    settings.set_active_account(username);
    settings.save_silent();

    let session = sessions.primary()?;

    let (mut notif_screen, task) =
        NotificationsScreen::new(session.client.clone(), session.user.clone());
    notif_screen.set_cross_account_priority(cross_account_priority);

    Some((Box::new(notif_screen), task.map(Message::Notifications)))
}

/// Handle logout - remove current account and switch or go to login.
///
/// Returns `Some((new_screen, task))` to switch to another account,
/// or `None` to indicate we should go to login screen.
pub fn handle_logout(
    sessions: &mut SessionManager,
    settings: &mut AppSettings,
) -> Option<(Box<NotificationsScreen>, Task<Message>)> {
    // Remove only the current account
    let current_username = sessions.primary().map(|s| s.username.clone());

    if let Some(username) = current_username {
        let _ = sessions.remove_account(&username);
        settings.remove_account(&username);
        settings.save_silent();
    }

    // If no accounts left, return None to signal login
    sessions.primary()?;

    // Switch to next available account
    let session = sessions.primary()?;
    settings.set_active_account(&session.username);
    settings.save_silent();

    let (notif_screen, task) =
        NotificationsScreen::new(session.client.clone(), session.user.clone());

    Some((Box::new(notif_screen), task.map(Message::Notifications)))
}
