use crate::app::model::Post;
use crate::app::api::ApiError;
use crate::core::config::Config;

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

/*#[derive(Debug, Clone)]
pub enum Message {
    // --- Startup ---
    Loaded(Config),
    LoadPosts(String),

    // --- Grid view ---
    LoadMorePosts,
    PostsLoaded(Vec<Post>),
    ThumbnailLoaded(u32, Handle),
    SearchInputChanged(String),
    SearchSubmitted,

    // --- Media ---
    ImageLoaded(u32, Handle),
    GifLoaded(u32, Vec<u8>),
    VideoLoaded(u32, Url),
    VideoPlayerMsg(VideoPlayerMessage),

    // --- Detail view ---
    ViewPost(u32),
    BackToGrid,
    AddTagToSearch(String),
    NegateTagFromSearch(String),

    // --- Tag Following ---
    CheckFollowedUpdates,
    FollowedUpdatesReceived(Vec<(String, Vec<Post>)>),
    FollowTag(String),
    NewFollowedTagChanged(String),
    AddFollowedTag,
    RemoveFollowedTag(String),
}*/

/// Messages to manage startup state
#[derive(Debug, Clone)]
pub enum StartupMessage {
    Loaded(Config)
}

/// Messages to manage the search state
#[derive(Debug, Clone)]
pub enum SearchMessage {
    LoadPosts(String),
    LoadMorePosts,
    PostsLoaded(Vec<Post>),
    SearchInputChanged(String),
    SearchSubmitted,
}

/// Manages post loading
#[derive(Debug, Clone)]
pub enum PostMessage {
    Load(String),
    Loaded(Vec<Post>),
    ThumbnailLoaded(u32, Handle),
    View(u32)
}

/// Messages to manage media display.
#[derive(Debug, Clone)]
pub enum MediaMessage {
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
    Save,
}

/// Messages to manage followed tags.
#[derive(Debug, Clone)]
pub enum FollowedMessage {
    NewTagAdded(String),
    AddTag,
    RemoveTag(String),
    UpdatesReceived(Result<Vec<String, Vec<Post>, ApiError)
}

/// Messages to manage view states (settings, followed, etc.)
#[derive(Debug, Clone)]
pub enum ViewMessage {
    ToggleSettings,
    WindowResized(u32),
}
