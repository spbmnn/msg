use std::collections::VecDeque;

use iced::widget::text_editor::Content;

use crate::core::config::Config;
use crate::core::model::{FollowedTag, Post};
use crate::core::store::PostStore;
use crate::gui::video_player::VideoPlayerWidget;

/// Values in settings fields
#[derive(Debug)]
pub struct Settings {
    /// Text in username input.
    pub username: String,
    /// Text in API key input.
    pub api_key: String,
    /// `Content` for the blacklist editor.
    pub blacklist_content: Content,
}

#[derive(Debug)]
pub struct UiState {
    // Maybe replace this with an enum?
    pub show_settings: bool,
    pub window_width: u32,
    pub window_height: u32,
}

#[derive(Debug)]
pub struct SearchState {
    /// Text in search bar.
    pub input: String,
    /// Current search query.
    pub query: String,
    /// Queue for thumbnail fetching.
    pub thumbnail_queue: VecDeque<u32>,
}

#[derive(Debug)]
pub struct FollowedState {
    pub new_followed_tag: String,
    pub new_followed_posts: Vec<(String, Vec<Post>)>,
    pub tags: Vec<FollowedTag>,
}

#[derive(Debug)]
pub struct App {
    pub settings: Settings,
    pub ui: UiState,
    pub search: SearchState,
    pub followed: FollowedState,
    pub config: Config,
    pub store: PostStore,

    /// Posts loaded in grid view.
    pub posts: Vec<Post>,
    /// None means grid view, Some(u32) is post ID.
    pub selected_post: Option<u32>,

    /// Shows "loading" screen.
    pub loading: bool,

    pub video_player: Option<VideoPlayerWidget>,
}
