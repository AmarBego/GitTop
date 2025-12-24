//! System tray management for GitTop.

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

#[derive(Debug, Clone)]
pub enum TrayCommand {
    ShowWindow,
    Quit,
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

        const ICON_BYTES: &[u8] = include_bytes!("../assets/images/GitTop-256x256.png");

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
                crate::platform::trim_memory();
            }
        }
    }
}
