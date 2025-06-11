use iced::{
    widget::{button, column, container, image, image::Handle, row, text, Button, Column},
    Element, Length,
};
use std::cmp::min;

use crate::{
    app::{
        message::{SearchMessage, ViewMessage},
        state::ViewMode,
        Message,
    },
    core::{
        model::{Post, Rating},
        store::PostStore,
    },
};

pub fn render<'a>(post: &Post, thumbnail: Option<&Handle>, width: f32) -> Element<'a, Message> {
    let rating_text = match post.rating {
        Rating::Safe => text("S").color(iced::Color::from_rgb(0.3, 0.9, 0.3)),
        Rating::Questionable => text("Q").color(iced::Color::from_rgb(0.9, 0.7, 0.2)),
        Rating::Explicit => text("E").color(iced::Color::from_rgb(1.0, 0.2, 0.2)),
    }
    .size(12);

    let thumbnail_size = width * (5.0 / 6.0);

    let preview: Element<_> = if let Some(img) = thumbnail {
        image(img.clone())
            .width(Length::Fixed(thumbnail_size))
            .height(Length::Fixed(thumbnail_size))
            .into()
    } else {
        container(text("No preview"))
            .width(Length::Fixed(thumbnail_size))
            .height(Length::Fixed(thumbnail_size))
            .center_x(Length::Fixed(thumbnail_size))
            .center_y(Length::Fixed(thumbnail_size))
            .into()
    };

    let ext = text(format!("{}", post.file.ext.clone().unwrap_or("".into()))).size(12);

    let meta = row![
        //text(format!("#{}", post.id)).size(12),
        text(format!("score: {}", post.score.total)).size(12),
        rating_text,
        ext,
    ]
    .spacing(8);

    let layout = column![preview, meta].spacing(4).padding(8);

    let height = width * (1.25);

    Button::new(layout)
        .on_press(Message::View(ViewMessage::Show(ViewMode::Detail(post.id))))
        .width(Length::Fixed(width))
        .height(Length::Fixed(height))
        .padding(4)
        .into()
}

pub fn grid_view<'a>(
    posts: &[Post],
    store: &PostStore,
    window_width: usize,
    posts_per_row: usize,
    tile_width: usize,
    load_more: bool,
) -> Column<'a, Message> {
    let mut grid = column![];

    let chunks = min(window_width / tile_width, posts_per_row);

    for chunk in posts.iter().collect::<Vec<_>>().chunks(chunks) {
        let mut r = row![];

        for post in chunk {
            let img = store.get_thumbnail(post.id);
            r = r.push(render(post, img, tile_width as f32));
        }

        grid = grid.push(container(r).center_x(Length::Fill).width(Length::Fill));
    }

    if load_more {
        grid = grid.push(
            button("load more")
                .on_press(Message::Search(SearchMessage::LoadMorePosts))
                .padding(8),
        );
    }

    grid.width(Length::Fill)
}
