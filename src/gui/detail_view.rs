use iced::theme::Palette;
use iced::widget::{button, column, container, image, row, scrollable, text};
use iced::widget::{Column, Row, Text};
use iced::{Alignment, Element, Length, Theme};
use iced_gif::{Frames, Gif};
use iced_video_player::VideoPlayer;
use tracing::error;

use crate::app::message::PostMessage;
use crate::core::model::Vote;
use crate::{
    app::message::{DetailMessage, FollowedMessage, MediaMessage, SearchMessage, ViewMessage},
    app::Message,
    core::{
        model::{Post, Rating},
        store::PostStore,
    },
};

use super::video_player::{VideoPlayerMessage, VideoPlayerWidget};

pub fn render_detail<'a>(
    post: &'a Post,
    store: &'a PostStore,
    video_player: &'a Option<VideoPlayerWidget>,
) -> Element<'a, Message> {
    let media_panel = column![
        button("back").on_press(Message::View(ViewMessage::ShowGrid)),
        text(format!("post #{}", post.id)),
        text(match post.rating {
            Rating::Safe => "Rating: Safe",
            Rating::Questionable => "Rating: Questionable",
            Rating::Explicit => "Rating: Explicit",
        }),
        text(format!("score: {}", post.score.total)),
        render_media(post, store, video_player),
        vote_bar(post, store),
        text(post.description.clone()),
    ];

    let info_panel = info_panel(&post);

    row![
        scrollable(media_panel.width(Length::FillPortion(9))),
        scrollable(info_panel.width(Length::FillPortion(3)))
    ]
    .into()
}

fn render_media<'a>(
    post: &Post,
    store: &'a PostStore,
    video_player: &'a Option<VideoPlayerWidget>,
) -> Element<'a, Message> {
    match post.get_type() {
        Some(crate::core::model::PostType::Image) => {
            if let Some(img) = store.get_image(post.id) {
                container(image(img).width(Length::Shrink).height(Length::Shrink))
                    .padding(16)
                    .into()
            } else {
                Text::new("Image not available").into()
            }
        }
        Some(crate::core::model::PostType::Gif) => {
            if let Some(_) = store.get_gif(post.id) {
                match store.gif_frames.get(&post.id) {
                    Some(frames) => Gif::new(frames).into(),
                    None => Text::new("Failed to parse GIF").into(),
                }
            } else {
                Text::new("GIF not available").into()
            }
        }
        Some(crate::core::model::PostType::Video) => {
            if let Some(vp) = video_player {
                vp.view()
                    .map(|msg| Message::Media(MediaMessage::VideoPlayerMsg(msg)))
                    .into()
            } else {
                Text::new("Cannot load video").into()
            }
        }
        _ => Text::new("Unsupported type").into(),
    }
}

fn info_panel<'a>(post: &'a Post) -> Column<'a, Message> {
    let mut panel = column![];

    for (category, tags) in post.tags.iter().filter(|(_, tags)| !tags.is_empty()) {
        let header = text(format!("{category}:")).size(16);

        let tag_list = column(tags.iter().map(|tag| render_tag(tag)).collect::<Vec<_>>())
            .spacing(8)
            .width(Length::Fill);

        panel = panel.push(header).push(tag_list).push(text(""));
    }

    panel.spacing(10)
}

fn render_tag(tag: &String) -> Element<'_, Message> {
    row![
        button("f")
            .on_press(Message::Followed(FollowedMessage::FollowTag(tag.clone())))
            .padding(4)
            .width(24),
        button("+")
            .on_press(Message::Detail(DetailMessage::AddTagToSearch(tag.clone())))
            .padding(4)
            .width(24),
        button("-")
            .on_press(Message::Detail(DetailMessage::NegateTagFromSearch(
                tag.clone()
            )))
            .padding(4)
            .width(24),
        button(text(tag))
            .on_press(Message::Search(SearchMessage::LoadPosts(tag.clone())))
            .padding(4),
    ]
    .into()
}

fn vote_bar<'a>(post: &'a Post, store: &'a PostStore) -> Row<'a, Message> {
    let vote_status = store.vote_for(post.id);
    let is_favorited = store.is_favorited(post.id);

    let upvote_button = button("↑")
        .on_press(match vote_status {
            Some(Vote::Upvote) => Message::Post(PostMessage::Vote(post.id, None)),
            _ => Message::Post(PostMessage::Vote(post.id, Some(Vote::Upvote))),
        })
        .style(move |theme: &Theme, _status| {
            let palette = theme.extended_palette();

            match vote_status {
                Some(Vote::Upvote) => {
                    button::Style::default().with_background(palette.success.strong.color)
                }
                _ => button::Style::default(),
            }
        });
    let downvote_button = button("↓")
        .on_press(match vote_status {
            Some(Vote::Downvote) => Message::Post(PostMessage::Vote(post.id, None)),
            _ => Message::Post(PostMessage::Vote(post.id, Some(Vote::Downvote))),
        })
        .style(move |theme: &Theme, _status| {
            let palette = theme.extended_palette();

            match vote_status {
                Some(Vote::Downvote) => {
                    button::Style::default().with_background(palette.danger.strong.color)
                }
                _ => button::Style::default(),
            }
        });

    let fav_button = button("fav")
        .on_press(Message::Post(PostMessage::Favorite(post.id)))
        .style(move |theme: &Theme, _status| {
            let palette = theme.extended_palette();

            if is_favorited {
                button::Style::default().with_background(palette.primary.strong.color)
            } else {
                button::Style::default()
            }
        });

    row![
        upvote_button,
        text(format!("{}", post.score.total)),
        downvote_button,
        fav_button
    ]
    .spacing(12)
}
