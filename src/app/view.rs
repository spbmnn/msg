use iced::{
    widget::{column, Row},
    Alignment, Element, Length, Theme,
};

use super::{state::ViewMode, App, Message};

// mod debug;
mod detail;
// mod dtext;
mod followed;
mod grid;
mod settings;

pub fn view(app: &App) -> Element<'_, Message> {
    let header: Row<Message> = match app.ui.view_mode {
        ViewMode::Grid => grid::search_bar(app),
        ViewMode::Detail(_) => detail::detail_bar(app),
        ViewMode::Settings => settings::settings_bar(app),
        ViewMode::Followed => followed::followed_bar(),
    }
    .spacing(8)
    .padding(8)
    .height(50)
    .align_y(Alignment::Center);

    // todo: add header bar in a similar manner
    let main_view = match app.ui.view_mode {
        ViewMode::Grid => grid::render_grid(app),
        ViewMode::Detail(_) => detail::render_detail(app),
        ViewMode::Settings => settings::render_settings(app),
        ViewMode::Followed => followed::render_followed(app),
    };

    //let debug_overlay = debug::render_debug_overlay(app);

    //let debug_row: Vec<Element<Message>> =
    //    vec![Space::with_width(Length::Fill).into(), debug_overlay];

    column![header, main_view]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

/// Sets the window title dynamically.
pub fn title(app: &App) -> String {
    let name = env!("CARGO_PKG_NAME");
    let version = env!("CARGO_PKG_VERSION");
    let window_title: String = match app.ui.view_mode {
        ViewMode::Grid => app.search.query.clone(),
        ViewMode::Followed => "Followed tags".into(),
        ViewMode::Settings => "Settings".into(),
        ViewMode::Detail(id) => format!("Post #{id}"),
    };

    return format!("{window_title} | {name} v{version}");
}

/// Gets the theme.
pub fn theme(app: &App) -> Theme {
    app.config.theme.get()
}
