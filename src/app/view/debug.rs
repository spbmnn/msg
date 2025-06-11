use iced::{
    widget::{column, container, text},
    Element, Length,
};

use crate::{
    app::{App, Message},
    core::model::PostType,
};

pub fn render_debug_overlay(app: &App) -> Element<'_, Message> {
    let mut debug_lines = vec![
        text(format!("Current view: {:?}", app.ui.view_mode)),
        text(format!(
            "Query: {}\nPage: {:?}",
            app.search.query, app.search.page
        )),
        text(format!("Posts cached: {}", app.store.posts.len())),
        text(format!("Thumbnails cached: {}", app.store.thumbnails.len())),
        text(format!("Images cached: {}", app.store.images.len())),
        text(format!("Gifs cached: {}", app.store.gifs.len())),
        text(format!("Videos cached: {}", app.store.videos.len())),
        text(format!("Thumbnail queue: {:?}", app.search.thumbnail_queue)),
        text(format!(
            "Undo: {:?}\nRedo:{:?}",
            app.ui.history.backwards, app.ui.history.forwards
        )),
        text(format!("Posts vec length: {}", app.posts.len())),
        text(format!("New followed posts: {:?}", app.followed_posts())),
    ];

    if let Some(auth) = &app.config.auth {
        debug_lines.push(text(format!("Auth: yes, as {}", auth.username)));
    } else {
        debug_lines.push(text("Auth: no"));
    }

    if let Some(id) = app.selected_post {
        debug_lines.push(text(format!("Selected post: {:?}", id)));
        if let Some(post) = app.store.get_post(id) {
            match post.get_type() {
                Some(PostType::Image) => debug_lines.push(text(format!(
                    "Thumbnail: {}\nSample: {}\nFullsize: {}",
                    app.store.has_thumbnail(id),
                    app.store.has_sample(id),
                    app.store.has_image(id)
                ))),
                Some(PostType::Gif) => debug_lines.push(text(format!(
                    "Thumbnail: {}\nGif: {}",
                    app.store.has_thumbnail(id),
                    app.store.has_gif(id)
                ))),
                Some(PostType::Video) => debug_lines.push(text(format!(
                    "Thumbnail: {}\nVideo: {}",
                    app.store.has_thumbnail(id),
                    app.store.has_video(id)
                ))),
                _ => {
                    debug_lines.push(text(format!("Thumbnail: {}", app.store.has_thumbnail(id))));
                }
            }
        }
    }

    container(
        column(
            debug_lines
                .into_iter()
                .map(Element::from)
                .collect::<Vec<Element<Message>>>(),
        )
        .spacing(4)
        .padding(8)
        .width(Length::Fill),
    )
    .into()
}
