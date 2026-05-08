//! FreeBSD-specific platform implementations.

use crate::settings::AppSettings;
use crate::tray::TrayCommand;
use crate::ui::App;
use iced::{Font, daemon, window};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Mutex, OnceLock};

// ============================================================================
// Cross-thread tray command bus (shared by tray and IPC listener)
// ============================================================================

struct CommandBus {
    tx: Sender<TrayCommand>,
    rx: Mutex<Receiver<TrayCommand>>,
}

static COMMAND_BUS: OnceLock<CommandBus> = OnceLock::new();

fn command_bus() -> &'static CommandBus {
    COMMAND_BUS.get_or_init(|| {
        let (tx, rx) = mpsc::channel();
        CommandBus {
            tx,
            rx: Mutex::new(rx),
        }
    })
}

pub(crate) fn command_sender() -> Sender<TrayCommand> {
    command_bus().tx.clone()
}

pub(crate) fn poll_command() -> Option<TrayCommand> {
    command_bus().rx.lock().ok()?.try_recv().ok()
}

/// Run the iced application using daemon mode.
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

/// Enable dark mode for system UI elements.
/// FreeBSD context menus follow GTK/Qt theme settings.
pub fn enable_dark_mode() {
    // Similar to Linux, GTK theming controls context menu appearance.
}

/// System tray implementation using ksni (pure-Rust StatusNotifierItem).
pub mod tray {
    use crate::tray::TrayCommand;
    use ksni::{self, Icon, Tray, menu::StandardItem};
    use std::sync::mpsc::Sender;

    struct GitTopTray {
        tx: Sender<TrayCommand>,
    }

    impl Tray for GitTopTray {
        fn id(&self) -> String {
            "gittop".into()
        }

        fn category(&self) -> ksni::Category {
            ksni::Category::ApplicationStatus
        }

        fn title(&self) -> String {
            "GitTop".into()
        }

        fn icon_name(&self) -> String {
            "gittop".into()
        }

        fn icon_pixmap(&self) -> Vec<Icon> {
            const ICON_BYTES: &[u8] = include_bytes!("../../assets/images/GitTop-256x256.png");

            if let Ok(icon) = Self::load_png_icon(ICON_BYTES) {
                vec![icon]
            } else {
                vec![]
            }
        }

        fn tool_tip(&self) -> ksni::ToolTip {
            ksni::ToolTip {
                title: "GitTop - GitHub Notifications".into(),
                ..Default::default()
            }
        }

        fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
            vec![
                StandardItem {
                    label: "Show GitTop".into(),
                    activate: Box::new(|tray: &mut Self| {
                        let _ = tray.tx.send(TrayCommand::ShowWindow);
                    }),
                    ..Default::default()
                }
                .into(),
                ksni::MenuItem::Separator,
                StandardItem {
                    label: "Quit".into(),
                    activate: Box::new(|tray: &mut Self| {
                        let _ = tray.tx.send(TrayCommand::Quit);
                    }),
                    ..Default::default()
                }
                .into(),
            ]
        }
    }

    impl GitTopTray {
        fn load_png_icon(bytes: &[u8]) -> Result<Icon, Box<dyn std::error::Error>> {
            use image::ImageReader;
            use std::io::Cursor;

            let img = ImageReader::new(Cursor::new(bytes))
                .with_guessed_format()?
                .decode()?
                .resize(32, 32, image::imageops::FilterType::Lanczos3)
                .into_rgba8();

            let (width, height) = img.dimensions();
            let raw = img.into_raw();

            // ksni expects ARGB format, convert from RGBA
            let argb: Vec<u8> = raw
                .chunks(4)
                .flat_map(|rgba| [rgba[3], rgba[0], rgba[1], rgba[2]])
                .collect();

            Ok(Icon {
                width: width as i32,
                height: height as i32,
                data: argb,
            })
        }
    }

    pub struct TrayManager {
        #[allow(dead_code)]
        handle: ksni::blocking::Handle<GitTopTray>,
    }

    impl TrayManager {
        pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
            use ksni::blocking::TrayMethods;

            let tray = GitTopTray {
                tx: super::command_sender(),
            };

            // Use blocking spawn API - spawns tray service in background thread
            let handle = tray.spawn()?;

            Ok(Self { handle })
        }

        pub fn poll_global_events() -> Option<TrayCommand> {
            super::poll_command()
        }
    }
}

/// Reduce memory footprint.
pub fn trim_memory() {
    // FreeBSD uses jemalloc by default.
    // Could potentially call jemalloc's purge functions.
    // For now, this is a no-op - the OS handles memory pressure.
}

/// Send a native FreeBSD notification via DBus.
///
/// Uses notify-rust which:
/// - Talks to the system notification daemon via DBus
/// - No polling required
/// - No background threads once fired
/// - Zero persistent memory cost
///
/// If `url` is provided, adds an "Open" action that opens the URL.
/// Works with any DBus-compatible notification daemon.
pub fn notify(title: &str, body: &str, url: Option<&str>) -> Result<(), notify_rust::error::Error> {
    use notify_rust::Notification;

    let mut notification = Notification::new();
    notification
        .summary(title)
        .body(body)
        .appname("GitTop")
        .timeout(5000); // 5 seconds

    // Add action if URL provided
    if let Some(url) = url {
        notification.action("open", "Open");

        // Show and handle action
        let handle = notification.show()?;

        let url_owned = url.to_string();
        std::thread::spawn(move || {
            handle.wait_for_action(|action| {
                if action == "open" || action == "default" {
                    let _ = open::that(&url_owned);
                }
            });
        });
        Ok(())
    } else {
        notification.show().map(|_| ())
    }
}

/// On-boot/autostart functionality for FreeBSD.
///
/// TODO: Investigate rc.d or user-level autostart mechanism.
pub mod on_boot {
    // Re-export the shared error type from the parent module
    pub use crate::platform::on_boot::OnBootError;

    /// Check if autostart is currently enabled.
    ///
    /// TODO: Investigate FreeBSD autostart mechanism
    pub fn is_enabled() -> bool {
        false
    }

    /// Enable autostart.
    ///
    /// TODO: Implement FreeBSD autostart
    pub fn enable() -> Result<(), OnBootError> {
        Err(OnBootError::NotSupported)
    }

    /// Disable autostart.
    ///
    /// TODO: Implement FreeBSD autostart
    pub fn disable() -> Result<(), OnBootError> {
        Err(OnBootError::NotSupported)
    }
}
