use iced::Element;
use iced::widget::{container, text};

use crate::ui::theme;

/// A styled card container for settings items.
pub fn setting_card<'a, Message>(content: impl Into<Element<'a, Message>>) -> Element<'a, Message>
where
    Message: 'a + Clone,
{
    let p = theme::palette();

    container(container(content).padding(14))
        .style(move |_| container::Style {
            background: Some(iced::Background::Color(p.bg_card)),
            border: iced::Border {
                radius: 8.0.into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .into()
}

/// Styled title for settings tabs.
pub fn tab_title<'a, Message>(title: &'static str) -> Element<'a, Message>
where
    Message: 'a + Clone,
{
    text(title)
        .size(20)
        .color(theme::palette().text_primary)
        .into()
}
