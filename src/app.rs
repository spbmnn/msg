use iced::Length::Fill;
use iced::{event, window, Event, Size, Subscription, Task};
use iced::{
    widget::{button, column, row, scrollable, text, text_input},
    Element,
};
use iced_gif::{Frames, Gif};
use iced_video_player::Video;
use tracing::{debug, error, info};

use iced::widget::image::Handle;
use url::Url;

use crate::core::api::fetch_posts;
use crate::core::config::Config;
use crate::core::media::{fetch_gif, fetch_image, fetch_preview, fetch_video, MediaError};
use crate::core::model::Post;
use crate::core::store::PostStore;
use crate::gui::detail_view::render_detail;
use crate::gui::post_tile::grid_view;

#[derive(Debug)]
pub enum Msg {
    Loading,
    Loaded(State),
}

#[derive(Debug)]
pub struct State {
    pub config: Config,
    pub posts: Vec<Post>,
    pub store: PostStore,
    pub current_tag: String,
    pub selected_post: Option<u32>,
    pub loading: bool,
    pub search_input: String,
}

#[derive(Debug, Clone)]
pub enum Message {
    Loaded(Config),
    LoadPosts(String),
    PostsLoaded(Vec<Post>),
    ThumbnailLoaded(u32, Handle),
    ImageLoaded(u32, Handle),
    GifLoaded(u32, Vec<u8>),
    VideoLoaded(u32, Url),
    ViewPost(u32),
    SearchInputChanged(String),
    SearchSubmitted,
    BackToGrid,
    Tick,
    EventOccurred(Event),
    Exit,
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
                        state.current_tag = query.clone();
                        return Task::perform(
                            crate::core::api::fetch_posts(query.clone(), 1),
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
                        state.store.insert_posts(posts.clone());
                        state.posts = posts;

                        let mut tasks: Vec<Task<Message>> = vec![];

                        for post in &state.posts {
                            if let Some(url) = &post.preview.url {
                                let post_id = post.id;
                                let url = url.clone();

                                tasks.push(Task::perform(fetch_preview(post_id, url), move |res| {
                                    match res {
                                        Ok(image) => Message::ThumbnailLoaded(post_id, image),
                                        Err(err) => {
                                            error!("thumbnail failed: {err}");
                                            Message::Tick
                                        }
                                    }
                                }))
                            }
                        }

                        return Task::batch(tasks);
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
                                        let url = post.file.url.clone().unwrap();
                                        commands.push(Task::perform(
                                            fetch_image(id, url, post.file.ext.clone().unwrap()),
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
                        let query = state.search_input.trim().to_string();
                        if !query.is_empty() {
                            info!("submitting search for {query}");
                            return Task::perform(
                                fetch_posts(query.clone(), 1),
                                move |res| match res {
                                    Ok(posts) => Message::PostsLoaded(posts),
                                    Err(err) => {
                                        error!("{err}");
                                        Message::Tick
                                    }
                                },
                            );
                        }
                    }
                    Message::BackToGrid => {
                        state.selected_post = None;
                    }
                    Message::EventOccurred(event) => {
                        if let Event::Window(window::Event::CloseRequested) = event {
                            info!("exiting...");

                            Config::save(&state.config);

                            return window::get_latest().and_then(window::close);
                        } else {
                            return Task::none();
                        }
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
                if let Some(selected_id) = state.selected_post {
                    // --- Detail view ---
                    if let Some(post) = state.store.get_post(selected_id) {
                        return render_detail(&post, &state.store);
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
                        .padding(8)
                ]
                .spacing(8);

                let mut images: Vec<Option<&Handle>> = vec![];

                for post in &state.posts {
                    let thumb = state.store.get_thumbnail(post.id);
                    images.push(thumb);
                }

                let content = if state.loading {
                    column![text("loading").size(16)]
                } else {
                    grid_view(&state.posts, images.as_slice(), 5).width(Fill)
                };

                column![search_bar, scrollable(content.padding(16))].into()
            }
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        event::listen().map(Message::EventOccurred)
    }

    pub fn new() -> (Self, Task<Message>) {
        debug!("creating new Msg");

        let mut app = Self::Loaded(State {
            config: Config::new(),
            current_tag: "fwankie".into(),
            ..Default::default()
        });

        let cmd = Task::perform(
            crate::core::api::fetch_posts(String::from("fav:homogoat"), 1),
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
        State {
            config: Config {
                username: None,
                api_key: None,
            },
            posts: vec![],
            store: PostStore::new(),
            current_tag: String::new(),
            search_input: String::new(),
            selected_post: None,
            loading: false,
        }
    }
}
