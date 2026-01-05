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

/// Focus existing GitTop window for single-instance support.
/// Uses EnumWindows to find and restore minimized windows.
pub fn focus_existing_window() {
    use windows::Win32::Foundation::{HWND, LPARAM};
    use windows::Win32::UI::WindowsAndMessaging::{
        EnumWindows, GetWindowTextW, IsIconic, IsWindowVisible, SW_RESTORE, SW_SHOW,
        SetForegroundWindow, ShowWindow,
    };

    // SAFETY: Callback only reads window properties, HWNDs valid during enumeration.
    unsafe extern "system" fn enum_callback(hwnd: HWND, _lparam: LPARAM) -> windows::core::BOOL {
        unsafe {
            if !IsWindowVisible(hwnd).as_bool() {
                return windows::core::BOOL::from(true);
            }

            // Use wide string API for proper Unicode support.
            let mut title = [0u16; 256];
            let len = GetWindowTextW(hwnd, &mut title);

            if len > 0 {
                let title_str = String::from_utf16_lossy(&title[..len as usize]);

                if title_str.contains("GitTop") {
                    if IsIconic(hwnd).as_bool() {
                        let _ = ShowWindow(hwnd, SW_RESTORE);
                    } else {
                        let _ = ShowWindow(hwnd, SW_SHOW);
                    }
                    let _ = SetForegroundWindow(hwnd);
                    return windows::core::BOOL::from(false);
                }
            }

            windows::core::BOOL::from(true)
        }
    }

    // SAFETY: EnumWindows with valid callback.
    unsafe {
        let _ = EnumWindows(Some(enum_callback), LPARAM(0));
    }
}

/// Enable dark mode for context menus via undocumented SetPreferredAppMode.
/// Widely used by Firefox/Chrome, degrades gracefully if API changes.
pub fn enable_dark_mode() {
    use windows::Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryA};
    use windows::core::PCSTR;

    const APPMODE_FORCEDARK: i32 = 2;
    type SetPreferredAppModeFn = unsafe extern "system" fn(i32) -> i32;

    // SAFETY: Load library, validate function pointer, call once.
    unsafe {
        let lib_name = CString::new("uxtheme.dll").unwrap();
        let lib = LoadLibraryA(PCSTR::from_raw(lib_name.as_ptr() as *const u8));

        if let Ok(handle) = lib {
            // Ordinal 135 = SetPreferredAppMode
            let func = GetProcAddress(handle, PCSTR::from_raw(135 as *const u8));

            if let Some(f) = func {
                let set_preferred_app_mode: SetPreferredAppModeFn = std::mem::transmute(f);
                set_preferred_app_mode(APPMODE_FORCEDARK);
            }
        }
    }
}

/// System tray implementation using tray-icon (native Windows APIs).
pub mod tray {
    use crate::tray::TrayCommand;
    use std::sync::OnceLock;
    use tray_icon::{
        Icon, TrayIcon, TrayIconBuilder, TrayIconEvent,
        menu::{Menu, MenuEvent, MenuId, MenuItem},
    };

    static MENU_IDS: OnceLock<MenuIds> = OnceLock::new();

    #[derive(Debug)]
    struct MenuIds {
        show: MenuId,
        quit: MenuId,
    }

    pub struct TrayManager {
        #[allow(dead_code)]
        tray: TrayIcon,
    }

    impl TrayManager {
        pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
            let show_item = MenuItem::new("Show GitTop", true, None);
            let quit_item = MenuItem::new("Quit", true, None);

            MENU_IDS
                .set(MenuIds {
                    show: show_item.id().clone(),
                    quit: quit_item.id().clone(),
                })
                .expect("TrayManager initialized twice");

            let menu = Menu::new();
            menu.append(&show_item)?;
            menu.append(&quit_item)?;

            let icon = Self::create_icon()?;
            let tray = TrayIconBuilder::new()
                .with_menu(Box::new(menu))
                .with_tooltip("GitTop - GitHub Notifications")
                .with_icon(icon)
                .build()?;

            Ok(Self { tray })
        }

        fn create_icon() -> Result<Icon, Box<dyn std::error::Error>> {
            use image::ImageReader;
            use std::io::Cursor;

            const ICON_BYTES: &[u8] = include_bytes!("../../assets/images/GitTop-256x256.png");

            let img = ImageReader::new(Cursor::new(ICON_BYTES))
                .with_guessed_format()?
                .decode()?
                .resize(32, 32, image::imageops::FilterType::Lanczos3)
                .into_rgba8();

            let (width, height) = img.dimensions();
            Icon::from_rgba(img.into_raw(), width, height).map_err(Into::into)
        }

        pub fn poll_global_events() -> Option<TrayCommand> {
            let command = Self::poll_menu_events();
            Self::drain_tray_icon_events();
            command
        }

        fn poll_menu_events() -> Option<TrayCommand> {
            let event = MenuEvent::receiver().try_recv().ok()?;
            let ids = MENU_IDS.get()?;

            [
                (&ids.show, TrayCommand::ShowWindow),
                (&ids.quit, TrayCommand::Quit),
            ]
            .into_iter()
            .find_map(|(id, cmd)| (event.id == *id).then_some(cmd))
        }

        fn drain_tray_icon_events() {
            while let Ok(event) = TrayIconEvent::receiver().try_recv() {
                if matches!(event, TrayIconEvent::Leave { .. }) {
                    super::trim_working_set();
                }
            }
        }
    }
}

/// Trim working set to reduce memory when minimized to tray.
pub fn trim_working_set() {
    use windows::Win32::System::ProcessStatus::EmptyWorkingSet;
    use windows::Win32::System::Threading::GetCurrentProcess;

    // SAFETY: GetCurrentProcess returns pseudo-handle, always valid.
    unsafe {
        let _ = EmptyWorkingSet(GetCurrentProcess());
    }
}

/// Send a native Windows toast notification.
/// Uses WinRT toasts - fire and forget, no resident memory.
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

    if let Some(url) = url {
        let url_owned = url.to_string();
        toast = toast.on_activated(move |_action| {
            let _ = open::that(&url_owned);
            Ok(())
        });
    }

    toast.show()
}

/// Autostart via HKCU\...\Run registry key. No elevated privileges needed.
pub mod on_boot {
    use windows::Win32::System::Registry::{
        HKEY, HKEY_CURRENT_USER, KEY_READ, KEY_WRITE, REG_SZ, REG_VALUE_TYPE, RegCloseKey,
        RegDeleteValueW, RegOpenKeyExW, RegQueryValueExW, RegSetValueExW,
    };
    use windows::core::HSTRING;

    pub use crate::platform::on_boot::OnBootError;

    const RUN_KEY_PATH: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
    const VALUE_NAME: &str = "GitTop";

    /// RAII wrapper - auto-closes registry key on drop to prevent leaks.
    struct RegKey(HKEY);

    impl RegKey {
        fn open(access: windows::Win32::System::Registry::REG_SAM_FLAGS) -> Option<Self> {
            let mut hkey = HKEY::default();
            let subkey = HSTRING::from(RUN_KEY_PATH);

            // SAFETY: Valid hive, subkey, and output pointer.
            let result =
                unsafe { RegOpenKeyExW(HKEY_CURRENT_USER, &subkey, Some(0), access, &mut hkey) };

            result.is_ok().then_some(Self(hkey))
        }
    }

    impl Drop for RegKey {
        fn drop(&mut self) {
            // SAFETY: Handle valid for struct lifetime, RegCloseKey infallible.
            let _ = unsafe { RegCloseKey(self.0) };
        }
    }

    pub fn is_enabled() -> bool {
        let Some(key) = RegKey::open(KEY_READ) else {
            return false;
        };

        let value_name = HSTRING::from(VALUE_NAME);
        let mut value_type = REG_VALUE_TYPE::default();
        let mut data_size: u32 = 0;

        // SAFETY: Query with null data buffer just checks existence.
        let result = unsafe {
            RegQueryValueExW(
                key.0,
                &value_name,
                Some(std::ptr::null()),
                Some(&mut value_type),
                None,
                Some(&mut data_size),
            )
        };

        result.is_ok()
    }

    pub fn enable() -> Result<(), OnBootError> {
        let exec_path = std::env::current_exe()
            .map_err(OnBootError::Io)?
            .to_string_lossy()
            .to_string();

        let quoted_path = format!("\"{}\"", exec_path);

        let key = RegKey::open(KEY_WRITE)
            .ok_or_else(|| OnBootError::CommandFailed("Failed to open registry key".to_string()))?;

        let value_name = HSTRING::from(VALUE_NAME);
        let wide_path: Vec<u16> = quoted_path
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();

        // SAFETY: Valid handle, null-terminated wide string, correct byte length.
        let data_bytes: &[u8] = unsafe {
            std::slice::from_raw_parts(wide_path.as_ptr() as *const u8, wide_path.len() * 2)
        };

        let result =
            unsafe { RegSetValueExW(key.0, &value_name, Some(0), REG_SZ, Some(data_bytes)) };

        if result.is_err() {
            return Err(OnBootError::CommandFailed(format!(
                "Failed to set registry value: {:?}",
                result
            )));
        }

        Ok(())
    }

    pub fn disable() -> Result<(), OnBootError> {
        let Some(key) = RegKey::open(KEY_WRITE) else {
            return Ok(()); // Key doesn't exist, nothing to disable
        };

        let value_name = HSTRING::from(VALUE_NAME);

        // SAFETY: Valid handle and value name.
        let result = unsafe { RegDeleteValueW(key.0, &value_name) };

        if result.is_err() {
            use windows::Win32::Foundation::ERROR_FILE_NOT_FOUND;
            let err_code = result.0 as u32;
            if err_code != ERROR_FILE_NOT_FOUND.0 {
                return Err(OnBootError::CommandFailed(format!(
                    "Failed to delete registry value: {:?}",
                    result
                )));
            }
        }

        Ok(())
    }
}
