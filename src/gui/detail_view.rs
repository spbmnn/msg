use iced::widget::{button, column, container, image, image::Handle, row, scrollable, text};
use iced::{Element, Length};
use iced_gif::{Frames, Gif};
use iced_video_player::VideoPlayer;

use crate::{
    app::Message,
    core::{
        model::{Post, Rating},
        store::PostStore,
    },
};

pub fn render_detail<'a>(post: &'a Post, store: &PostStore) -> Element<'a, Message> {
    let mut media_panel = column![
        button("back").on_press(Message::BackToGrid),
        text(format!("post #{}", post.id)),
        text(match post.rating {
            Rating::Safe => "Rating: Safe",
            Rating::Questionable => "Rating: Questionable",
            Rating::Explicit => "Rating: Explicit",
        }),
        text(format!("score: {}", post.score.total)),
    ];

    match post.file.ext.as_deref() {
        Some("gif") => {
            if let Some(data) = store.get_gif(post.id) {
                media_panel = media_panel.push(text("gif dont work yet oops"));
                //let frames = Frames::from_bytes(*data).clone().unwrap();
                //col = col.push(Gif::new(&frames));
            }
        }
        Some("webm") | Some("mp4") => {
            if let Some(url) = store.get_video(post.id) {
                media_panel = media_panel.push(text("video dont work yet oops"));
            }
        }
        _ => {
            if let Some(img) = store.get_image(post.id) {
                // MUST FIX: currently panics if image too large
                media_panel = media_panel.push(image(img.clone()));
            }
        }
    }

    let info_panel = info_panel(&post);

    row![media_panel, info_panel].into()
}

fn info_panel<'a>(post: &'a Post) -> Element<'a, Message> {
    let mut panel = column![];

    for (category, tags) in post.tags.iter().filter(|(_, tags)| !tags.is_empty()) {
        let header = text(format!("{category}:")).size(16);

        let tag_list = column(
            tags.iter()
                .map(|tag| {
                    container(text(tag))
                        .padding([4, 6])
                        .style(container::bordered_box)
                        .into()
                })
                .collect::<Vec<_>>(),
        )
        .spacing(8)
        .width(Length::Fill);

        panel = panel.push(header).push(tag_list).push(text(""));
    }

    scrollable(panel.spacing(10)).into()
}
