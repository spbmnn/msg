use std::collections::VecDeque;

use iced::widget::text_editor::Content;
use iced::Task;
use tracing::{debug, error, info};

use crate::app::message::SearchMessage;
use crate::core::api::fetch_posts;
use crate::core::config::Config;
use crate::core::model::{FollowedTag, Post};
use crate::core::store::{poststore_path, PostStore};
use crate::gui::video_player::VideoPlayerWidget;

use super::Message;

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
    pub view_mode: ViewMode,
    pub window_width: u32,
    pub window_height: u32,
    pub history: ViewHistory,
}

/// Stacks for back/forward buttons.
#[derive(Debug, Default)]
pub struct ViewHistory {
    /// "Undo" stack - push to when entering new state, pop when going back.
    pub backwards: Vec<ViewMode>,
    /// "Redo" stack - push to when undoing, pop when redoing.
    pub forwards: Vec<ViewMode>,
}

impl ViewHistory {
    /// Go backwards in the undo history, and store current state into redo.
    pub fn previous(&mut self, current: ViewMode) -> Option<ViewMode> {
        match self.backwards.pop() {
            Some(x) => {
                self.forwards.push(current);
                Some(x)
            }
            None => None,
        }
    }

    /// Go forwards in the redo history.
    pub fn next(&mut self, old_view: ViewMode) -> Option<ViewMode> {
        match self.forwards.pop() {
            Some(x) => {
                self.backwards.push(old_view);
                Some(x)
            }
            None => None,
        }
    }

    /// Store `view` in undo history and clear redo history.
    pub fn proceed(&mut self, old_view: ViewMode) {
        self.backwards.push(old_view);
        self.forwards.clear();
    }
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ViewMode {
    /// Grid view with search query.
    Grid(String),
    /// Detail view with post ID.
    Detail(u32),
    Settings,
    Followed,
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

impl App {
    pub fn new() -> (Self, Task<Message>) {
        debug!("creating new Msg");

        let search = SearchState {
            input: "order:rank".into(),
            query: "order:rank".into(),
            thumbnail_queue: VecDeque::new(),
        };

        let cache = if let Some(path) = poststore_path() {
            match PostStore::load_from(&path) {
                Ok(store) => {
                    info!("Loaded PostStore from {path:?}");
                    store
                }
                Err(err) => {
                    error!("Couldn't load PostStore from file: {err}");
                    PostStore::new()
                }
            }
        } else {
            info!("Cached PostStore not found");
            PostStore::new()
        };

        let app = Self {
            config: Config::new(),
            search: search,
            store: cache,
            ..Default::default()
        };

        let cmd = Task::perform(
            fetch_posts(None, String::from("order:rank"), None), // should fix
            move |res| match res {
                Ok(posts) => Message::Search(SearchMessage::PostsLoaded(posts)),
                Err(err) => {
                    error!("getting posts failed: {err}");
                    Message::Tick
                }
            },
        );

        (app, cmd)
    }
}

impl Default for App {
    fn default() -> Self {
        let config: Config = match Config::load() {
            Ok(config) => config,
            Err(_) => Config::default(),
        };
        let (username, api_key) = match config.auth {
            None => (String::new(), String::new()),
            Some(ref auth) => (auth.username.clone(), auth.api_key.clone()),
        };
        let followed_tags = config.followed_tags.clone();
        let blacklist = config.blacklist.rules.join("\n").clone();

        let store = PostStore::new();

        //let vote_path = data_dir().join("votes.mpk");
        //store.load_votes_from(&vote_path).unwrap_or_default();

        Self {
            settings: Settings {
                username: username,
                api_key: api_key,
                blacklist_content: Content::with_text(&blacklist).into(),
            },
            ui: UiState {
                view_mode: ViewMode::Grid(String::from("order:rank")),
                window_width: 480,
                window_height: 640,
                history: ViewHistory::default(),
            },
            search: SearchState {
                input: String::new(),
                query: String::new(),
                thumbnail_queue: VecDeque::new(),
            },
            followed: FollowedState {
                new_followed_tag: String::new(),
                new_followed_posts: Vec::new(),
                tags: followed_tags,
            },
            config: config,
            store: store,
            posts: Vec::new(),
            selected_post: None,
            loading: false,
            video_player: None,
            /*
            video_player: None,
            config: config,
            posts: vec![],
            followed_tags: followed_tags,
            store: PostStore::new(),
            current_tag: String::new(),
            search_input: String::new(),
            thumbnail_queue: VecDeque::new(),
            new_followed_tag: String::new(),
            new_followed_posts: Vec::new(),
            selected_post: None,
            loading: false,
            show_settings: false,
            settings_username: username,
            settings_api_key: api_key,
            blacklist_editor_content: Content::with_text(&blacklist).into(),
            window_height: 480,
            window_width: 640,*/
        }
    }
}
