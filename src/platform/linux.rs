//! Linux-specific platform implementations.

use crate::settings::AppSettings;
use crate::ui::App;
use iced::{Font, daemon, window};

/// Run the iced application using daemon mode.
/// Daemon mode allows the app to continue running with zero windows,
/// which is needed because Wayland doesn't support hiding windows.
pub fn run_app() -> iced::Result {
    daemon(App::new_for_daemon, App::update, App::view_for_daemon)
        .title(App::title_for_daemon)
        .theme(App::theme_for_daemon)
        .subscription(App::subscription)
        .antialiasing(true)
        .default_font(Font::DEFAULT)
        .run()
}

/// Build window settings for spawning from daemon.
pub fn build_initial_window_settings() -> (window::Id, iced::Task<crate::ui::app::Message>) {
    let settings = AppSettings::load();

    let size = iced::Size::new(
        if settings.window_width >= 100.0 {
            settings.window_width
        } else {
            800.0
        },
        if settings.window_height >= 100.0 {
            settings.window_height
        } else {
            640.0
        },
    );

    let position = match (settings.window_x, settings.window_y) {
        (Some(x), Some(y)) if x > -10000 && y > -10000 => {
            window::Position::Specific(iced::Point::new(x as f32, y as f32))
        }
        _ => window::Position::Centered,
    };

    let window_settings = window::Settings {
        size,
        position,
        platform_specific: window::settings::PlatformSpecific {
            application_id: "gittop".to_string(),
            ..Default::default()
        },
        ..Default::default()
    };

    let (id, task) = window::open(window_settings);
    (id, task.discard())
}

/// Focus an existing GitTop window from another process (single-instance detection).
/// Called when a second GitTop instance tries to launch.
///
/// Note: This is different from iced's `window::gain_focus()` used in app.rs,
/// which works within the same process for tray "Show" functionality.
pub fn focus_existing_window() {
    // On Linux, this requires display server IPC:
    // - X11: Could use xdotool or libX11 to find and activate window
    // - Wayland: Requires compositor-specific protocols (mostly unsupported)
    // For now, this is a no-op - users can click the tray icon instead.
}

/// Enable dark mode for system UI elements.
/// Linux context menus follow GTK/Qt theme settings.
pub fn enable_dark_mode() {
    // Linux context menus use GTK theming.
    // The theme is controlled by GTK_THEME or gsettings.
    // tray-icon/muda should respect the system theme.
}

/// Initialize the tray subsystem.
/// On Linux, tray-icon uses GTK which must be initialized before use.
pub fn init_tray() {
    // GTK must be initialized before tray-icon can create menus
    match gtk::init() {
        Ok(()) => {}
        Err(e) => {
            eprintln!("Failed to initialize GTK for tray icon: {:?}", e);
            eprintln!("Ensure GTK3 is installed: gtk3, libappindicator-gtk3");
        }
    }
}

/// Reduce memory footprint.
/// Uses malloc_trim on glibc systems.
pub fn trim_memory() {
    // On Linux with glibc, we can call malloc_trim to release memory
    // back to the OS. This is similar to EmptyWorkingSet on Windows.
    #[cfg(target_env = "gnu")]
    {
        // malloc_trim returns 1 if memory was released, 0 otherwise
        unsafe extern "C" {
            safe fn malloc_trim(pad: usize) -> i32;
        }
        malloc_trim(0);
    }
}

/// Send a native Linux notification via DBus.
///
/// Uses notify-rust which:
/// - Talks to the system notification daemon via DBus
/// - No polling required
/// - No background threads once fired
/// - Zero persistent memory cost
///
/// If `url` is provided, adds an "Open" action that opens the URL.
/// Works with: notify-osd, dunst, xfce4-notifyd, KDE, GNOME, etc.
pub fn notify(title: &str, body: &str, url: Option<&str>) -> Result<(), notify_rust::error::Error> {
    use notify_rust::Notification;

    let mut notification = Notification::new();
    notification
        .summary(title)
        .body(body)
        .appname("GitTop")
        .icon("gittop") // Uses icon from /usr/share/icons or ~/.local/share/icons
        .timeout(5000); // 5 seconds

    // Add action if URL provided
    if let Some(url) = url {
        notification.action("open", "Open");
        notification.hint(notify_rust::Hint::ActionIcons(true));

        // Show and handle action
        let handle = notification.show()?;

        // Clone URL for the closure
        let url_owned = url.to_string();
        // Spawn a thread to wait for action (non-blocking)
        std::thread::spawn(move || {
            handle.wait_for_action(|action| {
                if action == "open" || action == "default" {
                    let _ = open::that(&url_owned);
                }
            });
        });
        Ok(())
    } else {
        // Simple fire and forget
        notification.show().map(|_| ())
    }
}
