use iced::widget::{column, container, image, image::Handle, row, text, Button, Column};
use iced::Element;
use iced_gif::{Frames, Gif};
use iced_video_player::VideoPlayer;

use crate::{
    app::Message,
    core::{
        model::{Post, Rating},
        store::PostStore,
    },
};

pub fn render_detail<'a>(post: &Post, store: &PostStore) -> Element<'a, Message> {
    let mut col = column![
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
                col = col.push(text("gif dont work yet oops"));
                //let frames = Frames::from_bytes(*data).clone().unwrap();
                //col = col.push(Gif::new(&frames));
            }
        }
        Some("webm") | Some("mp4") => {
            if let Some(url) = store.get_video(post.id) {
                col = col.push(text("video dont work yet oops"));
            }
        }
        _ => {
            if let Some(img) = store.get_image(post.id) {
                col = col.push(image(img.clone()));
            }
        }
    }

    col.into()
}
