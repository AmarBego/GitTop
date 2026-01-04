//! Windows-specific platform implementations.

use crate::settings::AppSettings;
use crate::ui::App;
use iced::window::Position;
use iced::{Font, application};
use std::ffi::CString;

/// Run the iced application using normal application mode.
/// Windows supports Hidden mode properly, so no need for daemon.
pub fn run_app() -> iced::Result {
    let settings = AppSettings::load();

    let window_size = if settings.window_width >= 100.0 && settings.window_height >= 100.0 {
        iced::Size::new(settings.window_width, settings.window_height)
    } else {
        iced::Size::new(800.0, 640.0)
    };

    let window_position = match (settings.window_x, settings.window_y) {
        (Some(x), Some(y)) if x > -10000 && y > -10000 => {
            Position::Specific(iced::Point::new(x as f32, y as f32))
        }
        _ => Position::Centered,
    };

    let window_icon = load_window_icon();

    let window_settings = iced::window::Settings {
        size: window_size,
        position: window_position,
        icon: window_icon,
        ..Default::default()
    };

    application(App::new, App::update, App::view)
        .title(|app: &App| app.title())
        .theme(|app: &App| app.theme())
        .subscription(App::subscription)
        .window(window_settings)
        .antialiasing(true)
        .default_font(Font::DEFAULT)
        .exit_on_close_request(false)
        .run()
}

fn load_window_icon() -> Option<iced::window::Icon> {
    use std::io::Cursor;
    const ICON_BYTES: &[u8] = include_bytes!("../../assets/images/favicon-32x32.png");
    let img = image::ImageReader::new(Cursor::new(ICON_BYTES))
        .with_guessed_format()
        .ok()?
        .decode()
        .ok()?
        .to_rgba8();
    let (width, height) = img.dimensions();
    iced::window::icon::from_rgba(img.into_raw(), width, height).ok()
}
pub fn focus_existing_window() {
    use windows::Win32::Foundation::{HWND, LPARAM};
    use windows::Win32::UI::WindowsAndMessaging::{
        EnumWindows, GetWindowTextA, IsIconic, IsWindowVisible, SW_RESTORE, SW_SHOW,
        SetForegroundWindow, ShowWindow,
    };

    unsafe extern "system" fn enum_callback(hwnd: HWND, _lparam: LPARAM) -> windows::core::BOOL {
        unsafe {
            // Skip invisible windows
            if !IsWindowVisible(hwnd).as_bool() {
                return windows::core::BOOL::from(true);
            }

            // Get window title
            let mut title = [0u8; 256];
            let len = GetWindowTextA(hwnd, &mut title);

            if len > 0 {
                let title_str = std::str::from_utf8(&title[..len as usize]).unwrap_or("");

                // Check if this is a GitTop window
                if title_str.contains("GitTop") {
                    // Restore if minimized
                    if IsIconic(hwnd).as_bool() {
                        let _ = ShowWindow(hwnd, SW_RESTORE);
                    } else {
                        let _ = ShowWindow(hwnd, SW_SHOW);
                    }

                    // Bring to foreground
                    let _ = SetForegroundWindow(hwnd);

                    // Stop enumeration
                    return windows::core::BOOL::from(false);
                }
            }

            windows::core::BOOL::from(true)
        }
    }

    unsafe {
        let _ = EnumWindows(Some(enum_callback), LPARAM(0));
    }
}

/// Enable dark mode for Windows context menus (system tray).
/// Uses undocumented Windows API SetPreferredAppMode from uxtheme.dll.
pub fn enable_dark_mode() {
    // SetPreferredAppMode ordinal 135 in uxtheme.dll
    // 0 = Default, 1 = AllowDark, 2 = ForceDark, 3 = ForceLight, 4 = Max
    const APPMODE_FORCEDARK: i32 = 2;

    type SetPreferredAppModeFn = unsafe extern "system" fn(i32) -> i32;

    unsafe {
        let lib_name = CString::new("uxtheme.dll").unwrap();
        let lib = windows::Win32::System::LibraryLoader::LoadLibraryA(
            windows::core::PCSTR::from_raw(lib_name.as_ptr() as *const u8),
        );

        if let Ok(handle) = lib {
            // GetProcAddress with ordinal 135
            let func = windows::Win32::System::LibraryLoader::GetProcAddress(
                handle,
                windows::core::PCSTR::from_raw(135 as *const u8),
            );

            if let Some(f) = func {
                let set_preferred_app_mode: SetPreferredAppModeFn = std::mem::transmute(f);
                set_preferred_app_mode(APPMODE_FORCEDARK);
            }
        }
    }
}

/// Initialize the tray subsystem.
/// Windows doesn't require special initialization.
pub fn init_tray() {
    // No-op on Windows - tray-icon works without GTK
}

/// Aggressively trim the process working set to reduce memory footprint.
/// This moves pages to the page file, making the process appear to use less memory.
pub fn trim_working_set() {
    use windows::Win32::System::ProcessStatus::EmptyWorkingSet;
    use windows::Win32::System::Threading::GetCurrentProcess;

    unsafe {
        let _ = EmptyWorkingSet(GetCurrentProcess());
    }
}

/// Send a native Windows toast notification.
///
/// Uses WinRT toast notifications which:
/// - Don't require a background service
/// - Don't keep anything resident
/// - Fire and exit
///
/// If `url` is provided, clicking the notification opens that URL.
pub fn notify(
    title: &str,
    body: &str,
    url: Option<&str>,
) -> Result<(), tauri_winrt_notification::Error> {
    use tauri_winrt_notification::{Duration, Toast};

    let mut toast = Toast::new(Toast::POWERSHELL_APP_ID)
        .title(title)
        .text1(body)
        .duration(Duration::Short);

    // If URL provided, open it when notification is clicked
    if let Some(url) = url {
        let url_owned = url.to_string();
        toast = toast.on_activated(move |_action| {
            let _ = open::that(&url_owned);
            Ok(())
        });
    }

    // Fire and forget - no handles kept, no memory retained
    toast.show()
}

/// On-boot/autostart functionality for Windows.
///
/// TODO: Implement using Registry key or Startup folder.
/// - Registry: HKCU\Software\Microsoft\Windows\CurrentVersion\Run
/// - Startup folder: %APPDATA%\Microsoft\Windows\Start Menu\Programs\Startup
pub mod on_boot {
    // Re-export the shared error type from the parent module
    pub use crate::platform::on_boot::OnBootError;

    /// Check if autostart is currently enabled.
    ///
    /// TODO: Check Registry key HKCU\Software\Microsoft\Windows\CurrentVersion\Run
    pub fn is_enabled() -> bool {
        false
    }

    /// Enable autostart.
    ///
    /// TODO: Add Registry key to HKCU\Software\Microsoft\Windows\CurrentVersion\Run
    pub fn enable() -> Result<(), OnBootError> {
        Err(OnBootError::NotSupported)
    }

    /// Disable autostart.
    ///
    /// TODO: Remove Registry key from HKCU\Software\Microsoft\Windows\CurrentVersion\Run
    pub fn disable() -> Result<(), OnBootError> {
        Err(OnBootError::NotSupported)
    }
}
