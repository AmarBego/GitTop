use super::message::GeneralMessage;
use super::state::GeneralSettingsState;
use crate::settings::{AppSettings, IconTheme};
use crate::ui::theme;
use iced::Task;

pub fn update(
    state: &mut GeneralSettingsState,
    message: GeneralMessage,
    settings: &mut AppSettings,
) -> Task<GeneralMessage> {
    match message {
        GeneralMessage::ChangeTheme(new_theme) => {
            settings.theme = new_theme;
            theme::set_theme(new_theme);
            persist_settings(settings);
            tracing::info!(theme = %new_theme, "Theme updated");
            Task::none()
        }
        GeneralMessage::ToggleIconTheme(use_svg) => {
            settings.icon_theme = if use_svg {
                IconTheme::Svg
            } else {
                IconTheme::Emoji
            };
            persist_settings(settings);
            tracing::info!(use_svg, "Icon theme updated");
            Task::none()
        }
        GeneralMessage::ToggleMinimizeToTray(enabled) => {
            settings.minimize_to_tray = enabled;
            let _ = settings.save();
            tracing::info!(enabled, "Minimize-to-tray setting updated");
            Task::none()
        }
        GeneralMessage::SetNotificationFontScale(scale) => {
            let clamped = scale.clamp(0.8, 1.5);
            settings.notification_font_scale = clamped;
            theme::set_notification_font_scale(clamped);
            persist_settings(settings);
            tracing::debug!(scale = clamped, "Notification font scale updated");
            Task::none()
        }
        GeneralMessage::SetSidebarFontScale(scale) => {
            let clamped = scale.clamp(0.8, 1.5);
            settings.sidebar_font_scale = clamped;
            theme::set_sidebar_font_scale(clamped);
            persist_settings(settings);
            tracing::debug!(scale = clamped, "Sidebar font scale updated");
            Task::none()
        }
        GeneralMessage::SetSidebarWidth(width) => {
            let clamped = width.clamp(180.0, 400.0);
            settings.sidebar_width = clamped;
            persist_settings(settings);
            tracing::debug!(width = clamped, "Sidebar width updated");
            Task::none()
        }
        GeneralMessage::ToggleStartOnBoot(enabled) => {
            tracing::info!(enabled, "Start-on-boot toggle requested");
            // Perform the operation asynchronously and report result
            Task::perform(
                async move {
                    let result = if enabled {
                        crate::platform::on_boot::enable()
                    } else {
                        crate::platform::on_boot::disable()
                    };
                    result.map(|()| enabled).map_err(|e| e.to_string())
                },
                GeneralMessage::StartOnBootResult,
            )
        }
        GeneralMessage::StartOnBootResult(result) => {
            match result {
                Ok(new_state) => {
                    state.start_on_boot_enabled = new_state;
                    tracing::info!(enabled = new_state, "Start-on-boot setting updated");
                }
                Err(e) => {
                    tracing::error!(error = %e, "Failed to update start-on-boot setting");
                    // Re-query actual state to ensure UI reflects reality
                    state.start_on_boot_enabled = crate::platform::on_boot::is_enabled();
                }
            }
            Task::none()
        }
    }
}

fn persist_settings(settings: &mut AppSettings) {
    let _ = settings.save();
    crate::platform::trim_memory();
}
