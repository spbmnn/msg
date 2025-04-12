use iced::widget::{button, column, container, image, image::Handle, row, scrollable, text};
use iced::{Element, Length};
use iced_gif::{Frames, Gif};
use iced_video_player::VideoPlayer;
use tracing::error;

use crate::{
    app::Message,
    core::{
        model::{Post, Rating},
        store::PostStore,
    },
};

use super::video_player::VideoPlayerWidget;

pub fn render_detail<'a>(
    post: &'a Post,
    store: &PostStore,
    video_player: Option<&'a VideoPlayerWidget>,
) -> Element<'a, Message> {
    let media_panel = column![
        button("back").on_press(Message::BackToGrid),
        text(format!("post #{}", post.id)),
        text(match post.rating {
            Rating::Safe => "Rating: Safe",
            Rating::Questionable => "Rating: Questionable",
            Rating::Explicit => "Rating: Explicit",
        }),
        text(format!("score: {}", post.score.total)),
        render_media(post, store, video_player)
    ];

    let info_panel = info_panel(&post);

    row![media_panel, info_panel].into()
}

fn render_media<'a>(
    post: &Post,
    store: &PostStore,
    video_player: Option<&'a VideoPlayerWidget>,
) -> Element<'a, Message> {
    // --- IMAGE POSTS ---
    if let Some(img) = store.get_image(post.id) {
        return container(
            image(img.clone())
                .width(Length::Shrink)
                .height(Length::Shrink),
        )
        .padding(16)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into();
    }

    // --- GIF POSTS ---
    if let Some(data) = store.get_gif(post.id) {
        if let Ok(frames) = iced_gif::Frames::from_bytes(data.to_vec()) {
            return Gif::new(Box::leak(Box::new(frames))).into(); // this is atrocious
        }
    }

    // --- VIDEO POSTS ---
    if let Some(_url) = store.get_video(post.id) {
        if let Some(component) = video_player {
            return component.view().map(Message::VideoPlayerMsg);
        } else {
            return text("[video component not initialized]").into();
        }
    }

    return text("[media unavailable]").into();
}

fn info_panel<'a>(post: &'a Post) -> Element<'a, Message> {
    let mut panel = column![];

    for (category, tags) in post.tags.iter().filter(|(_, tags)| !tags.is_empty()) {
        let header = text(format!("{category}:")).size(16);

        let tag_list = column(tags.iter().map(|tag| render_tag(tag)).collect::<Vec<_>>())
            .spacing(8)
            .width(Length::Fill);

        panel = panel.push(header).push(tag_list).push(text(""));
    }

    scrollable(panel.spacing(10)).into()
}

fn render_tag(tag: &String) -> Element<'_, Message> {
    row![
        button("f")
            .on_press(Message::FollowTag(tag.clone()))
            .padding(4)
            .width(24),
        button("+")
            .on_press(Message::AddTagToSearch(tag.clone()))
            .padding(4)
            .width(24),
        button("-")
            .on_press(Message::NegateTagFromSearch(tag.clone()))
            .padding(4)
            .width(24),
        button(text(tag))
            .on_press(Message::LoadPosts(tag.clone()))
            .padding(4),
    ]
    .into()
}
