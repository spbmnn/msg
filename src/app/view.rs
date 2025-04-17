use iced::Element;

use super::{state::ViewMode, App, Message};

mod detail;
mod grid;
mod settings;

pub fn view(app: &App) -> Element<'_, Message> {
    match app.ui.view_mode {
        ViewMode::Grid => grid::render_grid(app),
        ViewMode::Detail(_) => detail::render_detail(app),
        ViewMode::Settings => settings::render_settings(app),
    }
}
