use iced::{
    widget::{column, row, scrollable, Row, Space},
    Element, Length, Renderer, Theme,
};

use super::{state::ViewMode, App, Message};

mod debug;
mod detail;
mod followed;
mod grid;
mod settings;

pub fn view(app: &App) -> Element<'_, Message> {
    let main_view = match app.ui.view_mode {
        ViewMode::Grid => grid::render_grid(app),
        ViewMode::Detail(_) => detail::render_detail(app),
        ViewMode::Settings => settings::render_settings(app),
        ViewMode::Followed => followed::render_followed(app),
    };

    let debug_overlay = debug::render_debug_overlay(app);

    let debug_row: Vec<Element<Message>> =
        vec![Space::with_width(Length::Fill).into(), debug_overlay];

    column![row(debug_row), scrollable(main_view)]
        .width(Length::Fill)
        .into()
}
