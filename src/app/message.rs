use iced::widget::image::Handle;
use iced::widget::text_editor::Action;
use url::Url;

use crate::core::config::Config;
use crate::core::model::Post;
use crate::gui::video_player::VideoPlayerMessage;

#[derive(Debug, Clone)]
pub enum Message {
    Tick,

    Startup(StartupMessage),
    Search(SearchMessage),
    Post(PostMessage),
    Media(MediaMessage),
    Detail(DetailMessage),
    Settings(SettingsMessage),
    Followed(FollowedMessage),
    View(ViewMessage),

    Exit,
}

/// Messages to manage startup state
#[derive(Debug, Clone)]
pub enum StartupMessage {
    Loaded(Config),
}

/// Messages to manage the search state
#[derive(Debug, Clone)]
pub enum SearchMessage {
    LoadPosts(String),
    LoadMorePosts,
    PostsLoaded(Vec<Post>),
    InputChanged(String),
    Submitted,
}

/// Manages post loading
#[derive(Debug, Clone)]
pub enum PostMessage {
    View(u32),
}

/// Messages to manage media display.
#[derive(Debug, Clone)]
pub enum MediaMessage {
    ThumbnailLoaded(u32, Handle),
    ImageLoaded(u32, Handle),
    GifLoaded(u32, Vec<u8>),
    VideoLoaded(u32, Url),
    VideoPlayerMsg(VideoPlayerMessage),
}

/// Messages for detail view.
#[derive(Debug, Clone)]
pub enum DetailMessage {
    AddTagToSearch(String),
    NegateTagFromSearch(String),
}

/// Messages to manage settings menu state.
#[derive(Debug, Clone)]
pub enum SettingsMessage {
    UsernameChanged(String),
    ApiKeyChanged(String),
    BlacklistEdited(Action),
    FollowFieldChanged(String),
    Save,
}

/// Messages to manage followed tags.
#[derive(Debug, Clone)]
pub enum FollowedMessage {
    CheckUpdates,
    UpdatesReceived(Vec<(String, Vec<Post>)>),
    AddTag,
    FollowTag(String),
    RemoveTag(String),
}

/// Messages to manage view states (settings, followed, etc.)
#[derive(Debug, Clone)]
pub enum ViewMessage {
    ShowSettings,
    ShowGrid,
    WindowResized(u32, u32),
}
