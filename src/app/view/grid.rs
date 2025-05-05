use crate::app::message::{FollowedMessage, SearchMessage, ViewMessage};
use crate::app::App;
use crate::app::Message;
use iced::widget::image::Handle;
use iced::{
    widget::{button, column, row, scrollable, text, text_input, Column, Row, Text},
    Element, Length,
};

pub fn render_grid(app: &App) -> Element<'_, Message> {
    let search_bar = row![
        text_input("search tags...", &app.search.input)
            .on_input(|input| Message::Search(SearchMessage::InputChanged(input)))
            .on_submit(Message::Search(SearchMessage::Submitted))
            .padding(8)
            .size(16),
        button("search")
            .on_press(Message::Search(SearchMessage::Submitted))
            .padding(8),
        button("settings")
            .on_press(Message::View(ViewMessage::ShowSettings))
            .padding(8),
        button("check")
            .on_press(Message::Followed(FollowedMessage::CheckUpdates))
            .padding(8)
    ]
    .spacing(8);

    let mut images: Vec<Option<&Handle>> = vec![];

    for post in &app.posts {
        let thumb = app.store.get_thumbnail(post.id);
        images.push(thumb);
    }

    let tile_width = 180;
    let max_columns = (app.ui.window_width / tile_width.max(1)).max(1);

    let mut content =
        crate::gui::post_tile::grid_view(&app.posts, images.as_slice(), max_columns as usize, true);

    column![
        search_bar,
        scrollable(content.padding(16)).width(Length::Fill)
    ]
    .width(Length::Fill)
    .into()
}
