use iced::widget::scrollable::Viewport;

#[derive(Debug, Clone)]
pub enum NotificationListMessage {
    ToggleGroup(usize),
    OnScroll(Viewport),
}
