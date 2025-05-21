use crate::app::message::{FollowedMessage, SearchMessage, ViewMessage};
use crate::app::App;
use crate::app::Message;
use iced::widget::image::Handle;
use iced::Alignment;
use iced::{
    widget::{button, column, row, scrollable, text, text_input, Column, Row, Text},
    Element, Length,
};

pub fn search_bar(app: &App) -> Row<'_, Message> {
    row![
        text_input("search tags...", &app.search.input)
            .on_input(|input| Message::Search(SearchMessage::InputChanged(input)))
            .on_submit(Message::Search(SearchMessage::Submitted))
            .padding(8)
            .size(16),
        button("favorites")
            .on_press(Message::Search(SearchMessage::GetFavorites))
            .padding(8),
        button("search")
            .on_press(Message::Search(SearchMessage::Submitted))
            .padding(8),
        button("settings")
            .on_press(Message::View(ViewMessage::ShowSettings))
            .padding(8),
        button("followed")
            .on_press(Message::Followed(FollowedMessage::CheckUpdates))
            .padding(8)
    ]
}

pub fn render_grid(app: &App) -> Element<'_, Message> {
    let mut images: Vec<Option<&Handle>> = vec![];

    for post in &app.posts {
        let thumb = app.store.get_thumbnail(post.id);
        images.push(thumb);
    }

    let tile_width = 180;
    let max_columns = (app.ui.window_width / tile_width.max(1)).max(1);

    let content =
        crate::gui::post_tile::grid_view(&app.posts, images.as_slice(), max_columns as usize, true);

    scrollable(content.padding(16)).into()
}
