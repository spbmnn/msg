use crate::app::state::ViewMode;
use crate::app::Message;
use crate::app::{message::*, App};
use crate::gui::post_tile::grid_view;
use iced::widget::image::Handle;
use iced::{
    widget::{button, column, row, scrollable, text, Row},
    Element, Length,
};

pub fn followed_bar<'a>() -> Row<'a, Message> {
    row![
        text("Followed tags").size(20).width(Length::Fill),
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

    for (tag, posts) in app.followed.new_followed_posts.clone() {
        let mut images: Vec<Option<&Handle>> = vec![];
        for post in &app.posts {
            let thumb = app.store.get_thumbnail(post.id);
            images.push(thumb);
        }
        content = content.push(column![
            text(tag),
            grid_view(
                &posts,
                images.as_slice(),
                app.ui.window_width as usize,
                app.config.view.posts_per_row,
                app.config.view.tile_width,
                false
            ),
        ]);
    }

    scrollable(content.padding(16).width(Length::Fill)).into()
}
