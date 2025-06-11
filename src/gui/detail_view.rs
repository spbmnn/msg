use chrono::{prelude::*, TimeDelta};

use iced::font::Weight;
use iced::widget::image::Handle;
use iced::widget::text::Shaping;
use iced::widget::{button, column, container, image, row, scrollable, text, Container};
use iced::widget::{Column, Row, Text};
use iced::{Alignment, Element, Length, Theme};
use iced_gif::Gif;

use crate::app::message::{PostMessage, ViewMessage};
use crate::app::state::ViewMode;
use crate::core::model::{Comment, Vote};
use crate::{
    app::message::{DetailMessage, FollowedMessage, MediaMessage},
    app::Message,
    core::{model::Post, store::PostStore},
};

use super::time_ago::relative_time_ago;
use super::video_player::VideoPlayerWidget;

pub fn render_detail<'a>(
    post: &'a Post,
    store: &'a PostStore,
    video_player: &'a Option<VideoPlayerWidget>,
) -> Element<'a, Message> {
    let mut media_panel = column![
        render_media(post, store, video_player),
        vote_bar(post, store),
        text(post.description.clone()).shaping(Shaping::Advanced),
    ];

    if let Some(comments) = store.get_comments(post.id) {
        media_panel = media_panel.push(render_comments(&comments));
    }

    let info_panel = info_panel(&post);

    row![
        scrollable(media_panel.width(Length::FillPortion(9)).padding(16)),
        scrollable(info_panel.width(Length::FillPortion(3)).padding(16))
    ]
    .into()
}

fn render_media<'a>(
    post: &Post,
    store: &'a PostStore,
    video_player: &'a Option<VideoPlayerWidget>,
) -> Container<'a, Message> {
    match post.get_type() {
        Some(crate::core::model::PostType::Image) => {
            get_image(store.get_image(post.id), store.get_sample(post.id))
        }
        Some(crate::core::model::PostType::Gif) => {
            if let Some(_) = store.get_gif(post.id) {
                match store.gif_frames.get(&post.id) {
                    Some(frames) => container(Gif::new(frames)),
                    None => container(Text::new("Failed to parse GIF")),
                }
            } else {
                container(Text::new("Gif loading..."))
            }
        }
        Some(crate::core::model::PostType::Video) => {
            if let Some(vp) = video_player {
                container(
                    vp.view()
                        .map(|msg| Message::Media(MediaMessage::VideoPlayerMsg(msg))),
                )
            } else {
                container(Text::new("Video loading..."))
            }
        }
        _ => container(Text::new("Unsupported type")),
    }
}

fn get_image<'a>(
    fullsize: Option<&'a Handle>,
    sample: Option<&'a Handle>,
) -> Container<'a, Message> {
    if let Some(img) = fullsize {
        return container(image(img));
    }
    if let Some(img) = sample {
        return container(image(img));
    }
    container(Text::new("Image loading..."))
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
            .on_press(Message::View(ViewMessage::Show(ViewMode::Grid(
                tag.clone(),
                Some(1)
            ))))
            .padding(4),
    ]
    .into()
}

fn render_comments<'a>(comments: &'a [Comment]) -> Column<'a, Message> {
    let all_comments = column(
        comments
            .iter()
            .map(|comment| render_comment(comment))
            .collect::<Vec<_>>(),
    );

    all_comments.spacing(10)
}

fn render_comment<'a>(comment: &'a Comment) -> Element<'a, Message> {
    let created_at: DateTime<Local> = DateTime::from(comment.created_at);
    let time_ago: TimeDelta = Local::now() - created_at;

    container(column![
        text(comment.creator_name.clone()).font(iced::font::Font {
            weight: Weight::Bold,
            ..Default::default()
        }),
        text(comment.body.clone()).shaping(Shaping::Advanced),
        text(relative_time_ago(time_ago)).size(10)
    ])
    .width(Length::Fill)
    .style(container::bordered_box)
    .padding(8)
    .into()
}

fn vote_bar<'a>(post: &'a Post, store: &'a PostStore) -> Row<'a, Message> {
    let vote_status = store.vote_for(post.id);
    let is_favorited = store.is_favorited(post.id);

    let upvote_button = button(text("↑").shaping(Shaping::Advanced))
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
    let downvote_button = button(text("↓").shaping(Shaping::Advanced))
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

    let fav_button = match is_favorited {
        true => button("unfav")
            .on_press(Message::Post(PostMessage::Favorite(post.id)))
            .style(move |theme: &Theme, _status| {
                let palette = theme.extended_palette();

                button::Style::default().with_background(palette.danger.strong.color)
            }),
        false => button("fav")
            .on_press(Message::Post(PostMessage::Favorite(post.id)))
            .style(move |theme: &Theme, _status| {
                let palette = theme.extended_palette();

                button::Style::default().with_background(palette.success.strong.color)
            }),
    };

    row![
        upvote_button,
        text(format!("{}", post.score.total)),
        downvote_button,
        fav_button
    ]
    .align_y(Alignment::Center)
    .spacing(12)
}
