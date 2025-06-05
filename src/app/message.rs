use iced::widget::image::Handle;
use iced::widget::text_editor::Action;
use url::Url;

use crate::app::state::{UiState, ViewMode};
use crate::core::config::MsgTheme;
use crate::core::model::{Comment, Post, Vote};
use crate::gui::video_player::VideoPlayerMessage;

#[derive(Debug, Clone)]
pub enum Message {
    Tick,
    Search(SearchMessage),
    Post(PostMessage),
    Media(MediaMessage),
    Detail(DetailMessage),
    Settings(SettingsMessage),
    Followed(FollowedMessage),
    View(ViewMessage),

    Exit,
}

/// Messages to manage the search state
#[derive(Debug, Clone)]
pub enum SearchMessage {
    LoadPosts(String),
    LoadMorePosts,
    PostsLoaded(Vec<Post>),
    InputChanged(String),
    Submitted,
    GetFavorites,
}

/// Manages post loading
#[derive(Debug, Clone)]
pub enum PostMessage {
    View(u32),
    Vote(u32, Option<Vote>),
    VoteResult(u32, Option<Vote>),
    Favorite(u32),
    FavoriteResult(u32, bool),
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
    CommentsLoaded(Vec<Comment>),
    CopyURL,
}

/// Messages to manage settings menu state.
#[derive(Debug, Clone)]
pub enum SettingsMessage {
    UsernameChanged(String),
    ApiKeyChanged(String),
    BlacklistEdited(Action),
    FollowFieldChanged(String),
    PPRChanged(usize),
    TileSizeChanged(usize),
    PurgeCache,
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
    Show(ViewMode),
    ShowWithoutProceed(ViewMode),
    WindowResized(u32, u32),
    /// For mouse back button, calls ShowGrid or Settings::Save accordingly
    Back,
    /// For mouse forward button
    Forward,
    UpdateTheme(MsgTheme),
}
