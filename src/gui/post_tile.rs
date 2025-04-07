use iced::{
    widget::{column, container, image, image::Handle, row, text, Button, Column},
    Element, Length,
};

use crate::{
    app::Message,
    core::model::{Post, Rating},
};

pub fn render<'a>(post: &Post, thumbnail: Option<&Handle>) -> Element<'a, Message> {
    let rating_text = match post.rating {
        Rating::Safe => text("S").color(iced::Color::from_rgb(0.3, 0.9, 0.3)),
        Rating::Questionable => text("Q").color(iced::Color::from_rgb(0.9, 0.7, 0.2)),
        Rating::Explicit => text("E").color(iced::Color::from_rgb(1.0, 0.2, 0.2)),
    }
    .size(12);

    let preview: Element<_> = if let Some(img) = thumbnail {
        image(img.clone())
            .width(Length::Fixed(150.0))
            .height(Length::Fixed(150.0))
            .into()
    } else {
        container(text("No preview"))
            .width(Length::Fixed(150.0))
            .height(Length::Fixed(150.0))
            .center_x(Length::Fixed(150.0))
            .center_y(Length::Fixed(150.0))
            .into()
    };

    let meta = row![
        text(format!("score: {}", post.score.total)).size(12),
        rating_text,
    ]
    .spacing(8);

    let layout = column![preview, meta].spacing(4).padding(8);

    Button::new(layout)
        .on_press(Message::ViewPost(post.id))
        .width(Length::Shrink)
        .padding(4)
        .into()
}

pub fn grid_view<'a>(
    posts: &[Post],
    images: &[Option<&Handle>],
    width: usize,
) -> Column<'a, Message> {
    let mut grid = column![];

    for chunk in posts.iter().zip(images).collect::<Vec<_>>().chunks(width) {
        let mut r = row![];

        for (post, img) in chunk {
            r = r.push(render(post, **img));
        }

        grid = grid.push(r);
    }

    grid
}
