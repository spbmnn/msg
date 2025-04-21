use crate::app::Message;
use crate::app::{message::*, App};
use crate::gui::post_tile::grid_view;
use iced::widget::image::Handle;
use iced::Length;
use iced::{
    widget::{button, column, row, scrollable, text, text_input},
    Element,
};
use tracing::debug;

pub fn render_followed(app: &App) -> Element<'_, Message> {
    let search_bar = row![
        text("followed tags").size(16),
        button("search")
            .on_press(Message::Search(SearchMessage::Submitted))
            .padding(8),
        button("settings")
            .on_press(Message::View(ViewMessage::ShowSettings))
            .padding(8),
        button("back")
            .on_press(Message::View(ViewMessage::ShowGrid))
            .padding(8)
    ]
    .spacing(8);

    let tile_width = 180;
    let max_columns = (app.ui.window_width / tile_width.max(1)).max(1);

    let mut content = column![];

    for (tag, posts) in app.followed.new_followed_posts.clone() {
        let mut images: Vec<Option<&Handle>> = vec![];
        for post in &app.posts {
            let thumb = app.store.get_thumbnail(post.id);
            images.push(thumb);
        }
        content = content.push(column![
            text(tag),
            grid_view(&posts, images.as_slice(), max_columns as usize),
        ]);
    }

    column![
        search_bar,
        scrollable(content.padding(16)).width(Length::Fill)
    ]
    .width(Length::Fill)
    .into()
}
