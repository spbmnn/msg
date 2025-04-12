use std::cell::RefCell;
use std::collections::VecDeque;

use iced::widget::text_editor::{Action, Content};
use iced::Length;
use iced::{event, window, Event, Subscription, Task};
use iced::{
    widget::{button, column, row, scrollable, text, text_input},
    Element,
};
use tracing::{debug, error, info, warn};

use iced::widget::image::Handle;
use url::Url;

use crate::core::api::fetch_posts;
use crate::core::config::{Auth, Config};
use crate::core::media::{fetch_gif, fetch_image, fetch_preview, fetch_video};
use crate::core::model::{FollowedTag, Post};
use crate::core::store::PostStore;
use crate::core::{blacklist, followed};
use crate::gui::blacklist_editor::{self, BlacklistEditor};
use crate::gui::detail_view::render_detail;
use crate::gui::post_tile::grid_view;
use crate::gui::settings;
use crate::gui::video_player::{VideoPlayerMessage, VideoPlayerWidget};

pub enum Msg {
    Loading,
    Loaded(State),
}

pub struct State {
    pub video_player: Option<VideoPlayerWidget>,
    pub config: Config,
    pub posts: Vec<Post>,
    pub store: PostStore,
    pub current_tag: String,
    pub selected_post: Option<u32>,
    pub loading: bool,
    pub search_input: String,
    pub thumbnail_queue: VecDeque<u32>,
    pub followed_tags: Vec<FollowedTag>,
    pub new_followed_tag: String,
    new_followed_posts: Vec<(String, Vec<Post>)>,
    show_settings: bool,
    settings_username: String,
    settings_api_key: String,
    blacklist_editor_content: Content,
    window_width: u32,
    window_height: u32,
}

#[derive(Debug, Clone)]
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

    // --- Settings view ---
    ShowSettings,
    SaveSettings,
    UsernameChanged(String),
    ApiKeyChanged(String),
    BlacklistEdited(Action),

    // --- Tag Following ---
    CheckFollowedUpdates,
    FollowedUpdatesReceived(Vec<(String, Vec<Post>)>),
    FollowTag(String),
    NewFollowedTagChanged(String),
    AddFollowedTag,
    RemoveFollowedTag(String),

    // --- Utilities ---
    Tick,
    Exit,
    WindowResized(u32, u32),
}

impl Msg {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match self {
            Msg::Loading => {
                match message {
                    Message::Loaded(config) => {
                        *self = Msg::Loaded(State {
                            config: config,
                            ..State::default()
                        });
                    }
                    _ => {}
                }
                Task::none()
            }
            Msg::Loaded(state) => {
                match message {
                    Message::LoadPosts(query) => {
                        state.loading = true;
                        state.posts.clear();
                        state.selected_post = None;
                        state.current_tag = query.clone();
                        state.search_input = query.clone(); // in case it's being called from another function
                        return Task::perform(
                            crate::core::api::fetch_posts(
                                query.clone(),
                                None,
                                state.config.auth.clone(),
                            ),
                            move |res| match res {
                                Ok(posts) => Message::PostsLoaded(posts),
                                Err(err) => {
                                    error!("Error fetching posts: {err}");
                                    Message::Tick
                                }
                            },
                        );
                    }
                    Message::LoadMorePosts => {
                        let before_id = state.posts.iter().map(|p| p.id).min();
                        state.loading = true;
                        let query = state.search_input.clone();
                        return Task::perform(
                            crate::core::api::fetch_posts(
                                query.clone(),
                                before_id,
                                state.config.auth.clone(),
                            ),
                            move |res| match res {
                                Ok(posts) => Message::PostsLoaded(posts),
                                Err(err) => {
                                    error!("Error fetching posts: {err}");
                                    Message::Tick
                                }
                            },
                        );
                    }
                    Message::PostsLoaded(posts) => {
                        state.loading = false;
                        let filtered = posts
                            .into_iter()
                            .filter(|p| {
                                !blacklist::is_blacklisted(p, &state.config.blacklist.rules)
                            })
                            .collect::<Vec<_>>();
                        state.store.insert_posts(filtered.clone());
                        let new_posts = filtered
                            .into_iter()
                            .filter(|p| !state.posts.iter().any(|existing| existing.id == p.id))
                            .collect::<Vec<_>>();
                        state.posts.extend(new_posts.clone());

                        for post in &new_posts {
                            if post.preview.url.is_some() {
                                state.thumbnail_queue.push_back(post.id);
                            }
                        }
                    }
                    Message::AddTagToSearch(tag) => {
                        state.selected_post = None;
                        state.current_tag.push_str(&(" ".to_owned() + &tag));
                        state.search_input = state.current_tag.clone();
                        state.loading = true;
                        return Task::perform(
                            fetch_posts(state.current_tag.clone(), None, state.config.auth.clone()),
                            move |res| match res {
                                Ok(posts) => Message::PostsLoaded(posts),
                                Err(err) => {
                                    error!("Couldn't load posts: {err}");
                                    Message::BackToGrid
                                }
                            },
                        );
                    }
                    Message::NegateTagFromSearch(tag) => {
                        state.selected_post = None;
                        state.current_tag.push_str(&(" -".to_owned() + &tag));
                        state.search_input = state.current_tag.clone();
                        state.loading = true;
                        return Task::perform(
                            fetch_posts(state.current_tag.clone(), None, state.config.auth.clone()),
                            move |res| match res {
                                Ok(posts) => {
                                    if posts.len() == 0 {
                                        Message::Tick
                                    } else {
                                        Message::PostsLoaded(posts)
                                    }
                                }
                                Err(err) => {
                                    error!("Couldn't load posts: {err}");
                                    Message::BackToGrid
                                }
                            },
                        );
                    }
                    Message::ViewPost(id) => {
                        state.selected_post = Some(id);
                        info!("selected post {id}");

                        let mut commands = vec![];

                        if let Some(post) = state.store.get_post(id) {
                            match post.file.ext.as_deref() {
                                Some("gif") => {
                                    if !state.store.has_gif(id) {
                                        let url = post.file.url.clone().unwrap();
                                        commands.push(Task::perform(
                                            fetch_gif(id, url),
                                            move |res| match res {
                                                Ok(gif) => Message::GifLoaded(id, gif),
                                                Err(err) => {
                                                    error!("gif failed: {err}");
                                                    Message::Tick
                                                }
                                            },
                                        ));
                                    }
                                }
                                Some("webm") | Some("mp4") => {
                                    if !state.store.has_video(id) {
                                        let url = post.file.url.clone().unwrap();
                                        commands.push(Task::perform(
                                            fetch_video(id, url, post.file.ext.clone().unwrap()),
                                            move |res| match res {
                                                Ok(url) => Message::VideoLoaded(id, url),
                                                Err(err) => {
                                                    error!("video failed: {err}");
                                                    Message::Tick
                                                }
                                            },
                                        ));
                                    }
                                }
                                _ => {
                                    if !state.store.has_image(id) {
                                        let file = post.file.clone();
                                        commands.push(Task::perform(
                                            fetch_image(id, file),
                                            move |res| match res {
                                                Ok(handle) => Message::ImageLoaded(id, handle),
                                                Err(err) => {
                                                    error!("image failed: {err}");
                                                    Message::Tick
                                                }
                                            },
                                        ));
                                    }
                                }
                            }
                        }
                        return Task::batch(commands);
                    }
                    Message::ThumbnailLoaded(id, image) => {
                        state.store.insert_thumbnail(id, image);
                    }
                    Message::ImageLoaded(id, image) => {
                        state.store.insert_image(id, image);
                    }
                    Message::GifLoaded(id, gif) => {
                        state.store.insert_gif(id, gif);
                    }
                    Message::VideoLoaded(id, video) => {
                        state.store.insert_video(id, video);
                    }
                    Message::SearchInputChanged(query) => {
                        state.search_input = query;
                        return Task::none();
                    }
                    Message::SearchSubmitted => {
                        state.posts.clear();
                        let query = state.search_input.trim().to_string();
                        state.current_tag = query.clone();
                        if !query.is_empty() {
                            info!("submitting search for {query}");
                            return Task::perform(
                                fetch_posts(query.clone(), None, state.config.auth.clone()),
                                move |res| match res {
                                    Ok(posts) => Message::PostsLoaded(posts),
                                    Err(err) => {
                                        error!("{err}");
                                        Message::LoadPosts(String::new())
                                    }
                                },
                            );
                        }
                    }
                    Message::BackToGrid => {
                        state.selected_post = None;
                    }
                    Message::Exit => {
                        info!("exiting...");

                        match Config::save(&state.config) {
                            Ok(()) => (),
                            Err(err) => {
                                error!("Couldn't save config: {err}");
                            }
                        }

                        return window::get_latest().and_then(window::close);
                    }
                    Message::WindowResized(width, height) => {
                        state.window_width = width;
                        state.window_height = height;
                    }
                    Message::VideoPlayerMsg(msg) => {
                        if let Some(player) = &mut state.video_player {
                            return player.update(msg);
                        }
                    }
                    Message::Tick => {
                        if let Some(post_id) = state.thumbnail_queue.pop_front() {
                            if let Some(post) = state.posts.iter().find(|p| p.id == post_id) {
                                if let Some(url) = &post.preview.url {
                                    return Task::perform(
                                        fetch_preview(post_id, url.clone()),
                                        move |res| match res {
                                            Ok(thumb) => Message::ThumbnailLoaded(post_id, thumb),
                                            Err(err) => {
                                                error!("Failed to fetch thumbnail for {post_id}: {err}");
                                                Message::Tick
                                            }
                                        },
                                    );
                                }
                            }
                        }
                    }
                    Message::UsernameChanged(username) => {
                        state.settings_username = username;
                    }
                    Message::ApiKeyChanged(api_key) => {
                        state.settings_api_key = api_key;
                    }
                    Message::ShowSettings => {
                        state.show_settings = true;
                    }
                    Message::BlacklistEdited(action) => {
                        state.blacklist_editor_content.perform(action);
                    }
                    Message::SaveSettings => {
                        debug!("Saving settings.");
                        state.config.auth = Some(Auth {
                            username: state.settings_username.clone(),
                            api_key: state.settings_api_key.clone(),
                        });

                        let blacklist = state
                            .blacklist_editor_content
                            .text()
                            .lines()
                            .map(str::trim)
                            .filter(|line| !line.is_empty())
                            .map(str::to_string)
                            .collect::<Vec<String>>();
                        state.config.blacklist.rules = blacklist;

                        state.config.followed_tags = state.followed_tags.clone();

                        if let Err(err) = state.config.save() {
                            warn!("Failed to save config: {err}");
                        }

                        state.show_settings = false;
                    }
                    Message::NewFollowedTagChanged(new_tag) => {
                        state.new_followed_tag = new_tag;
                    }
                    Message::AddFollowedTag => {
                        let tag = state.new_followed_tag.trim();
                        if !tag.is_empty() && !state.followed_tags.iter().any(|f| f.tag == tag) {
                            state.followed_tags.push(FollowedTag {
                                tag: tag.to_string(),
                                last_seen_post_id: None,
                            });

                            state.config.followed_tags = state.followed_tags.clone();
                            let _ = state.config.save();
                        }
                        state.new_followed_tag.clear();
                    }
                    Message::RemoveFollowedTag(tag) => {
                        state.followed_tags.retain(|f| f.tag != tag);

                        state.config.followed_tags = state.followed_tags.clone();
                        let _ = state.config.save();
                    }
                    Message::CheckFollowedUpdates => {
                        let tags = state.followed_tags.clone();
                        let auth = state.config.auth.clone();
                        return Task::perform(
                            followed::check_for_updates(tags, auth),
                            move |res| match res {
                                Ok(stuff) => Message::FollowedUpdatesReceived(stuff),
                                Err(err) => {
                                    error!("Couldn't check followed tags.");
                                    Message::Tick
                                }
                            },
                        );
                    }
                    Message::FollowedUpdatesReceived(updates) => {
                        for (tag, posts) in &updates {
                            let max_id = posts.iter().map(|p| p.id).max();
                            if let Some(latest) = max_id {
                                if let Some(f) =
                                    state.followed_tags.iter_mut().find(|f| f.tag == *tag)
                                {
                                    f.last_seen_post_id = Some(latest);
                                }
                            }
                        }

                        state.new_followed_posts = updates;

                        let _ = state.config.save();
                    }
                    _ => {}
                }
                Task::none()
            }
        }
    }

    pub fn view(&self) -> Element<Message> {
        match self {
            Msg::Loading => text("loading").into(),
            Msg::Loaded(state) => {
                if state.show_settings {
                    // --- Settings view ---
                    let settings_view = {
                        settings::render_settings(
                            &state.settings_username,
                            &state.settings_api_key,
                            &state.blacklist_editor_content,
                            &state.followed_tags,
                            &state.new_followed_tag,
                        )
                    };
                    return settings_view;
                }

                if let Some(selected_id) = state.selected_post {
                    // --- Detail view ---
                    if let Some(post) = state.store.get_post(selected_id) {
                        return render_detail(&post, &state.store, state.video_player.as_ref());
                    }
                }

                let search_bar = row![
                    text_input("search tags...", &state.search_input)
                        .on_input(Message::SearchInputChanged)
                        .on_submit(Message::SearchSubmitted)
                        .padding(8)
                        .size(16),
                    button("search")
                        .on_press(Message::SearchSubmitted)
                        .padding(8),
                    button("settings")
                        .on_press(Message::ShowSettings)
                        .padding(8),
                    button("check")
                        .on_press(Message::CheckFollowedUpdates)
                        .padding(8)
                ]
                .spacing(8);

                let mut images: Vec<Option<&Handle>> = vec![];

                for post in &state.posts {
                    let thumb = state.store.get_thumbnail(post.id);
                    images.push(thumb);
                }

                let tile_width = 180;
                let max_columns = (state.window_width / tile_width.max(1)).max(1);

                let content = if state.loading {
                    column![text("loading").size(16)]
                } else {
                    grid_view(&state.posts, images.as_slice(), max_columns as usize)
                        .width(Length::Fill)
                };

                column![
                    search_bar,
                    scrollable(content.padding(16)).width(Length::Fill)
                ]
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
            }
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        use iced::time;
        use std::time::Duration;

        let mut subs = vec![];

        subs.push(event::listen_with(|event, _, _| match event {
            Event::Window(window::Event::CloseRequested) => Some(Message::Exit),
            Event::Window(window::Event::Resized(size)) => Some(Message::WindowResized(
                size.width.floor() as u32,
                size.height.floor() as u32,
            )),
            _ => None,
        }));

        match self {
            Msg::Loading => (),
            Msg::Loaded(state) => {
                if !state.thumbnail_queue.is_empty() {
                    subs.push(time::every(Duration::from_millis(50)).map(|_| Message::Tick));
                }
            }
        }

        Subscription::batch(subs)
    }

    pub fn new() -> (Self, Task<Message>) {
        debug!("creating new Msg");

        let app = Self::Loaded(State {
            config: Config::new(),
            current_tag: "fav:homogoat".into(),
            search_input: "fav:homogoat".into(),
            ..Default::default()
        });

        let cmd = Task::perform(
            crate::core::api::fetch_posts(String::from("fav:homogoat"), None, None), // should fix
            move |res| match res {
                Ok(posts) => Message::PostsLoaded(posts),
                Err(err) => {
                    error!("getting posts failed: {err}");
                    Message::Tick
                }
            },
        );

        (app, cmd)
    }

    pub fn title(&self) -> String {
        format!("MSG")
    }
}

impl Default for State {
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

        State {
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
            window_width: 640,
        }
    }
}
