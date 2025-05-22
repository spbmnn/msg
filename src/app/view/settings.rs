use crate::app::message::SettingsMessage;
use crate::app::App;
use crate::app::Message;
use iced::{
    widget::{button, row, text, Row},
    Element, Length,
};

pub fn settings_bar(_app: &App) -> Row<'_, Message> {
    row![
        text("Settings").size(20).width(Length::Fill),
        button("save").on_press(Message::Settings(SettingsMessage::Save)),
    ]
}

pub fn render_settings(app: &App) -> Element<'_, Message> {
    crate::gui::settings::render_settings(
        &app.settings.username,
        &app.settings.api_key,
        &app.settings.blacklist_content,
        &app.followed.tags,
        &app.store,
        &app.followed.new_followed_tag,
        &app.config.theme,
    )
}
