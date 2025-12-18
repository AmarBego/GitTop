#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

//! GitTop - A beautiful native GitHub notification manager
//! No browser engine required. Pure Rust. Pure performance.

mod github;
mod settings;
mod tray;
mod ui;

use iced::{application, Font, Size};
use single_instance::SingleInstance;
use ui::App;

/// Mutex name for single instance detection
const SINGLE_INSTANCE_MUTEX: &str = "GitTop-SingleInstance-Mutex-7a8b9c0d";

fn main() -> iced::Result {
    // Check for existing instance
    let instance = SingleInstance::new(SINGLE_INSTANCE_MUTEX).unwrap();
    
    if !instance.is_single() {
        // Another instance is running - try to focus it and exit
        #[cfg(windows)]
        {
            focus_existing_window();
        }
        return Ok(());
    }

    // Enable dark mode for context menus on Windows
    #[cfg(windows)]
    {
        enable_dark_mode();
    }

    // Initialize tray icon on main thread (required for macOS)
    // The tray must be kept alive for the duration of the app
    let _tray = tray::TrayManager::new().ok();

    application(App::new, App::update, App::view)
        .title(|app: &App| app.title())
        .theme(|app: &App| app.theme())
        .subscription(App::subscription)
        .window_size(Size::new(420.0, 640.0))
        .antialiasing(true)
        .default_font(Font::DEFAULT)
        .exit_on_close_request(false)
        .run()
}

/// Find and focus an existing GitTop window on Windows
#[cfg(windows)]
fn focus_existing_window() {
    use windows::Win32::Foundation::{HWND, LPARAM};
    use windows::Win32::UI::WindowsAndMessaging::{
        EnumWindows, GetWindowTextA, IsIconic, IsWindowVisible, SetForegroundWindow, ShowWindow,
        SW_RESTORE, SW_SHOW,
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

/// Enable dark mode for Windows context menus (system tray)
/// Uses undocumented Windows API SetPreferredAppMode from uxtheme.dll
#[cfg(windows)]
fn enable_dark_mode() {
    use std::ffi::CString;
    
    // SetPreferredAppMode ordinal 135 in uxtheme.dll
    // 0 = Default, 1 = AllowDark, 2 = ForceDark, 3 = ForceLight, 4 = Max
    const APPMODE_FORCEDARK: i32 = 2;
    
    type SetPreferredAppModeFn = unsafe extern "system" fn(i32) -> i32;
    
    unsafe {
        let lib_name = CString::new("uxtheme.dll").unwrap();
        let lib = windows::Win32::System::LibraryLoader::LoadLibraryA(
            windows::core::PCSTR::from_raw(lib_name.as_ptr() as *const u8)
        );
        
        if let Ok(handle) = lib {
            // GetProcAddress with ordinal 135
            let func = windows::Win32::System::LibraryLoader::GetProcAddress(
                handle,
                windows::core::PCSTR::from_raw(135 as *const u8)
            );
            
            if let Some(f) = func {
                let set_preferred_app_mode: SetPreferredAppModeFn = std::mem::transmute(f);
                set_preferred_app_mode(APPMODE_FORCEDARK);
            }
        }
    }
}
