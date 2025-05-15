use iced::{
    widget::{column, container, text},
    Element, Length,
};

use crate::app::{App, Message};

pub fn render_debug_overlay(app: &App) -> Element<'_, Message> {
    let mut debug_lines = vec![
        text(format!("Current view: {:?}", app.ui.view_mode)),
        text(format!("Query: {}", app.search.query)),
        text(format!("Thumbnails cached: {}", app.store.thumbnails.len())),
        text(format!("Images cached: {}", app.store.images.len())),
        text(format!("Gifs cached: {}", app.store.gifs.len())),
        text(format!("Videos cached: {}", app.store.videos.len())),
        text(format!(
            "Thumbnail queue: {}",
            app.search.thumbnail_queue.len()
        )),
    ];

    if let Some(auth) = &app.config.auth {
        debug_lines.push(text(format!("Auth: yes, as {}", auth.username)));
    } else {
        debug_lines.push(text("Auth: no"));
    }

    if let Some(id) = app.selected_post {
        debug_lines.push(text(format!("Selected post: {}", id)));
        if app.store.get_thumbnail(id).is_some() {
            debug_lines.push(text("Thumbnail: loaded"));
        } else {
            debug_lines.push(text("Thumbnail: missing"));
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
