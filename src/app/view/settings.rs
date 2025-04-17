use crate::app::App;
use crate::app::Message;
use iced::Element;

pub fn render_settings(app: &App) -> Element<'_, Message> {
    crate::gui::settings::render_settings(
        &app.settings.username,
        &app.settings.api_key,
        &app.settings.blacklist_content,
        &app.followed.tags,
        &app.followed.new_followed_tag,
    )
}
