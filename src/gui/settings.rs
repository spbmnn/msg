use iced::{
    widget::{
        button, button::danger, column, container, pick_list, row, scrollable, text, text_editor,
        text_editor::Content, text_input,
    },
    Element, Length,
};

use crate::{
    app::{
        message::{FollowedMessage, SettingsMessage, ViewMessage},
        Message,
    },
    core::{config::MsgTheme, model::FollowedTag, store::PostStore},
};

pub fn render_settings<'a>(
    username: &'a str,
    api_key: &'a str,
    blacklist_content: &'a Content,
    followed_tags: &'a Vec<FollowedTag>,
    cache: &'a PostStore,
    new_followed_tag: &'a str,
    current_theme: &'a MsgTheme,
) -> Element<'a, Message> {
    let username_input = text_input("username", username)
        .on_input(|user| Message::Settings(SettingsMessage::UsernameChanged(user)));
    let api_key_input = text_input("api key", api_key)
        .on_input(|key| Message::Settings(SettingsMessage::ApiKeyChanged(key)))
        .secure(true);
    let blacklist_editor = text_editor(blacklist_content)
        .on_action(|bl| Message::Settings(SettingsMessage::BlacklistEdited(bl)))
        .height(300);

    let cache_info = cache_info(cache);
    let appearance_settings = appearance_settings(current_theme);

    scrollable(
        column![
            text("e621 login").size(16),
            username_input,
            api_key_input,
            blacklist_editor,
            followed_tag_settings(followed_tags, new_followed_tag),
            text("cache info").size(16),
            cache_info,
            text("appearance").size(16),
            appearance_settings,
        ]
        .spacing(12)
        .padding(24),
    )
    .into()
}

fn cache_info<'a>(cache: &'a PostStore) -> Element<'a, Message> {
    let info_lines = column![
        text(format!("Posts stored: {}", cache.posts.len())),
        text(format!("Thumbnails cached: {}", cache.thumbnails.len())),
        text(format!("Images cached: {}", cache.images.len())),
        text(format!("Gifs cached: {}", cache.gifs.len())),
        text(format!("Gif framesets stored: {}", cache.gif_frames.len())),
        text(format!("Videos cached: {}", cache.videos.len())),
        text(format!("Favorites stored: {}", cache.favorites.len())),
        text(format!("Votes stored: {}", cache.votes.len())),
        button("Purge cache")
            .on_press(Message::Settings(SettingsMessage::PurgeCache))
            .style(danger),
        text("Purging cache removes downloads for all posts, except any you have favorited.")
            .size(10),
    ];

    container(info_lines.spacing(4).padding(8)).into()
}

fn followed_tag_settings<'a>(
    followed_tags: &'a Vec<FollowedTag>,
    new_followed_tag: &'a str,
) -> Element<'a, Message> {
    let followed_tag_input = text_input("new tag", new_followed_tag)
        .on_input(|field| Message::Settings(SettingsMessage::FollowFieldChanged(field)))
        .on_submit(Message::Followed(FollowedMessage::AddTag))
        .width(Length::Fill);

    let tag_buttons = row(followed_tags.iter().map(|tag| {
        row![
            text(tag.to_string()),
            button("x").on_press(Message::Followed(FollowedMessage::RemoveTag(
                tag.to_string()
            )))
        ]
        .spacing(4)
        .padding(6)
        .into()
    }))
    .spacing(8)
    .wrap();

    column![text("followed tags:"), tag_buttons, followed_tag_input]
        .spacing(12)
        .into()
}

fn appearance_settings<'a>(current_theme: &'a MsgTheme) -> Element<'a, Message> {
    let theme_options = [MsgTheme::Dark, MsgTheme::Light];

    let settings = column![row![
        text("Theme"),
        pick_list(theme_options, Some(current_theme), |theme| {
            Message::View(ViewMessage::UpdateTheme(theme))
        })
    ]];

    container(settings.spacing(4).padding(8))
        .style(container::bordered_box)
        .into()
}
