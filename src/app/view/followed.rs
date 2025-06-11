use crate::app::state::ViewMode;
use crate::app::Message;
use crate::app::{message::*, App};
use crate::gui::post_tile::grid_view;
use iced::{
    widget::{button, column, row, scrollable, text, Row},
    Element, Length,
};

pub fn followed_bar<'a>() -> Row<'a, Message> {
    row![
        text("Followed tags").size(20).width(Length::Fill),
        button("mark as seen")
            .on_press(Message::Followed(FollowedMessage::ClearSeenPosts))
            .padding(8),
        button("settings")
            .on_press(Message::View(ViewMessage::Show(ViewMode::Settings)))
            .padding(8),
        button("back")
            .on_press(Message::View(ViewMessage::Back))
            .padding(8)
    ]
}

pub fn render_followed(app: &App) -> Element<'_, Message> {
    let mut content = column![];

    for (tag, posts) in app.followed.new_followed_posts.iter() {
        if posts.is_empty() {
            content = content.push(column![text(tag), text("no new posts").size(12)]);
        } else {
            content = content.push(column![
                text(tag),
                grid_view(
                    &posts,
                    &app.store,
                    app.ui.window_width as usize,
                    app.config.view.posts_per_row,
                    app.config.view.tile_width,
                    false
                ),
            ]);
        }
    }

    scrollable(content.padding(16).width(Length::Fill)).into()
}
