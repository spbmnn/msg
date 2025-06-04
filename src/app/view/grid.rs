use crate::app::message::{FollowedMessage, SearchMessage, ViewMessage};
use crate::app::state::ViewMode;
use crate::app::App;
use crate::app::Message;
use crate::core::config::Auth;
use crate::core::model::Post;
use iced::widget::image::Handle;
use iced::Length;
use iced::{
    widget::{button, row, scrollable, text_input, Row},
    Element,
};

pub fn search_bar(app: &App) -> Row<'_, Message> {
    row![
        text_input("search tags...", &app.search.input)
            .on_input(|input| Message::Search(SearchMessage::InputChanged(input)))
            .on_submit(Message::Search(SearchMessage::Submitted))
            .padding(8)
            .size(16),
        button("favorites")
            .on_press(Message::View(ViewMessage::Show(ViewMode::Grid(format!(
                "fav:{}",
                app.config
                    .auth
                    .as_ref()
                    .unwrap_or(&Auth::default())
                    .username
            )))))
            .padding(8),
        button("search")
            .on_press(Message::Search(SearchMessage::Submitted))
            .padding(8),
        button("settings")
            .on_press(Message::View(ViewMessage::Show(ViewMode::Settings)))
            .padding(8),
        button("followed")
            .on_press(Message::Followed(FollowedMessage::CheckUpdates))
            .padding(8)
    ]
}

pub fn render_grid<'a>(app: &'a App, query: &'a str) -> Element<'a, Message> {
    let mut images: Vec<Option<&Handle>> = vec![];
    let posts: Vec<Post> = app
        .store
        .get_results(query)
        .unwrap_or(&Vec::new())
        .iter()
        .filter_map(|&id| app.store.get_post(id).cloned())
        .collect();

    for post in &posts {
        let thumb = app.store.get_thumbnail(post.id);
        images.push(thumb);
    }

    let content = crate::gui::post_tile::grid_view(
        &posts,
        images.as_slice(),
        app.ui.window_width as usize,
        app.config.view.posts_per_row,
        app.config.view.tile_width,
        true,
    );

    scrollable(content.padding(16)).width(Length::Fill).into()
}
