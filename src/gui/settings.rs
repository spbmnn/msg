use std::cell::RefCell;

use iced::{
    widget::{button, column, row, text, text_editor, text_editor::Content, text_input},
    Element, Length,
};

use crate::{app::Message, core::model::FollowedTag};

use super::blacklist_editor::BlacklistEditor;

pub fn render_settings<'a>(
    username: &'a str,
    api_key: &'a str,
    blacklist_content: &'a Content,
    followed_tags: &'a Vec<FollowedTag>,
    new_followed_tag: &'a str,
) -> Element<'a, Message> {
    let username_input = text_input("username", username).on_input(Message::UsernameChanged);
    let api_key_input = text_input("api key", api_key)
        .on_input(Message::ApiKeyChanged)
        .secure(true);
    let blacklist_editor = text_editor(blacklist_content)
        .on_action(Message::BlacklistEdited)
        .height(300);

    let save_button = button("save").on_press(Message::SaveSettings);

    column![
        text("e621 login").size(20),
        username_input,
        api_key_input,
        blacklist_editor,
        followed_tag_settings(followed_tags, new_followed_tag),
        save_button
    ]
    .spacing(12)
    .padding(24)
    .into()
}

fn followed_tag_settings<'a>(
    followed_tags: &'a Vec<FollowedTag>,
    new_followed_tag: &'a str,
) -> Element<'a, Message> {
    let followed_tag_input = text_input("new tag", new_followed_tag)
        .on_input(Message::NewFollowedTagChanged)
        .on_submit(Message::AddFollowedTag)
        .width(Length::Fill);

    let tag_buttons = row(followed_tags.iter().map(|tag| {
        row![
            text(tag.to_string()),
            button("✕").on_press(Message::RemoveFollowedTag(tag.to_string()))
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
