use crate::app::message::{
    DetailMessage, FollowedMessage, MediaMessage, Message, PostMessage, SearchMessage,
    SettingsMessage, StartupMessage, ViewMessage,
};
use crate::app::state::App;
use crate::core::api::fetch_posts;
use crate::core::blacklist;
use crate::core::config::Auth;
use crate::core::media::{fetch_gif, fetch_image, fetch_video};
use crate::core::model::{FollowedTag, Post};
use iced::Task;
use tracing::{debug, error, info, warn};

pub fn update(app: &mut App, message: Message) -> Task<Message> {
    match message {
        Message::Startup(msg) => update_startup(app, msg),
        Message::Search(msg) => update_search(app, msg),
        Message::Post(msg) => update_post(app, msg),
        Message::Media(msg) => update_media(app, msg),
        Message::Detail(msg) => update_detail(app, msg),
        Message::Settings(msg) => update_settings(app, msg),
        Message::Followed(msg) => update_followed(app, msg),
        Message::View(msg) => update_view(app, msg),
        Message::Tick => Task::none(),
        Message::Exit => Task::none(),
    }
}

fn update_startup(app: &mut App, msg: StartupMessage) -> Task<Message> {
    Task::none()
}

fn update_search(app: &mut App, msg: SearchMessage) -> Task<Message> {
    match msg {
        SearchMessage::LoadPosts(query) => {
            app.loading = true;
            app.posts.clear();
            app.selected_post = None;
            app.search.query = query.clone();
            return Task::perform(
                crate::core::api::fetch_posts(query.clone(), None, app.config.auth.clone()),
                move |res| match res {
                    Ok(posts) => Message::Post(PostMessage::Loaded(posts)),
                    Err(err) => {
                        error!("Error fetching posts: {err}");
                        Message::Tick
                    }
                },
            );
        }
        SearchMessage::LoadMorePosts => {
            let before_id = app.posts.iter().map(|p| p.id).min();
            app.loading = true;
            let query = app.search.input.clone();
            return Task::perform(
                crate::core::api::fetch_posts(query.clone(), before_id, app.config.auth.clone()),
                move |res| match res {
                    Ok(posts) => Message::Post(PostMessage::Loaded(posts)),
                    Err(err) => {
                        error!("Error fetching posts: {err}");
                        Message::Tick
                    }
                },
            );
        }
        SearchMessage::PostsLoaded(posts) => {
            app.loading = false;
            let filtered = posts
                .into_iter()
                .filter(|p| !blacklist::is_blacklisted(p, &app.config.blacklist.rules))
                .collect::<Vec<Post>>();
            app.store.insert_posts(filtered.clone());
            let new_posts = filtered
                .into_iter()
                .filter(|p| !app.posts.iter().any(|existing| existing.id == p.id))
                .collect::<Vec<Post>>();
            app.posts.extend(new_posts.clone());

            for post in &new_posts {
                if post.preview.url.is_some() {
                    app.search.thumbnail_queue.push_back(post.id);
                }
            }
        }
        SearchMessage::SearchInputChanged(text) => {
            app.search.input = text;
        }
        SearchMessage::SearchSubmitted => {
            app.posts.clear();
            let query = app.search.input.trim().to_string();
            app.search.query = query.clone();
            if !query.is_empty() {
                info!("Submitting search for {query}");
                return Task::perform(
                    fetch_posts(query.clone(), None, app.config.auth.clone()),
                    move |res| match res {
                        Ok(posts) => Message::Search(SearchMessage::PostsLoaded(posts)),
                        Err(err) => {
                            error!("{err}");
                            Message::Search(SearchMessage::LoadPosts(String::new()))
                        }
                    },
                );
            }
        }
    }
    Task::none()
}

fn update_post(app: &mut App, msg: PostMessage) -> Task<Message> {
    match msg {
        PostMessage::View(id) => {
            app.selected_post = Some(id);
            info!("Selected post {id}");

            // Build task batch
            let mut commands = vec![];

            if let Some(post) = app.store.get_post(id) {
                // TODO: Deal with .swfs for compatiblity.
                // *Maaaaaaaybe* ruffle support? Doubt it.
                match post.file.ext.as_deref() {
                    Some("gif") => {
                        if !app.store.has_gif(id) {
                            let url = post.file.url.clone().unwrap();
                            commands.push(Task::perform(
                                fetch_gif(id, url),
                                move |res| match res {
                                    Ok(gif) => Message::Media(MediaMessage::GifLoaded(id, gif)),
                                    Err(err) => {
                                        error!("Gif {id} failed: {err}");
                                        Message::Tick
                                    }
                                },
                            ));
                        }
                    }
                    Some("webm") | Some("mp4") => {
                        if !app.store.has_video(id) {
                            if !app.store.has_video(id) {
                                let url = post.file.url.clone().unwrap();
                                commands.push(Task::perform(
                                    fetch_video(id, url, post.file.ext.clone().unwrap()),
                                    move |res| match res {
                                        Ok(url) => {
                                            Message::Media(MediaMessage::VideoLoaded(id, url))
                                        }
                                        Err(err) => {
                                            error!("Video {id} failed: {err}");
                                            Message::Tick
                                        }
                                    },
                                ));
                            }
                        }
                    }
                    Some("swf") => {}
                    _ => {
                        if !app.store.has_image(id) {
                            let file = post.file.clone();
                            commands.push(Task::perform(
                                fetch_image(id, file),
                                move |res| match res {
                                    Ok(handle) => {
                                        Message::Media(MediaMessage::ImageLoaded(id, handle))
                                    }
                                    Err(err) => {
                                        error!("Image {id} failed: {err}");
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
    }
}

fn update_media(app: &mut App, msg: MediaMessage) -> Task<Message> {
    match msg {
        MediaMessage::ThumbnailLoaded(id, handle) => app.store.insert_thumbnail(id, handle),
        MediaMessage::ImageLoaded(id, handle) => app.store.insert_image(id, handle),
        MediaMessage::GifLoaded(id, gif) => app.store.insert_gif(id, gif),
        MediaMessage::VideoLoaded(id, url) => app.store.insert_video(id, url),
        MediaMessage::VideoPlayerMsg(message) => {
            if let Some(player) = &mut app.video_player {
                return player.update(message);
            }
        }
    }
    Task::none()
}

fn update_detail(app: &mut App, msg: DetailMessage) -> Task<Message> {
    match msg {
        DetailMessage::AddTagToSearch(tag) => {
            app.selected_post = None;
            app.search.query.push_str(&(" ".to_owned() + &tag));
            app.search.input = app.search.query.clone();
            app.loading = true;

            return Task::perform(
                fetch_posts(app.search.query.clone(), None, app.config.auth.clone()),
                move |res| match res {
                    Ok(posts) => {
                        if posts.len() == 0 {
                            Message::Tick
                        } else {
                            Message::Search(SearchMessage::PostsLoaded(posts))
                        }
                    }
                    Err(err) => {
                        error!("Couldn't load posts: {err}");
                        Message::Detail(DetailMessage::BackToGrid)
                    }
                },
            );
        }
        DetailMessage::NegateTagFromSearch(tag) => {
            app.selected_post = None;
            app.search.query.push_str(&(" -".to_owned() + &tag));
            app.search.input = app.search.query.clone();
            app.loading = true;

            return Task::perform(
                fetch_posts(app.search.query.clone(), None, app.config.auth.clone()),
                move |res| match res {
                    Ok(posts) => {
                        if posts.len() == 0 {
                            Message::Tick
                        } else {
                            Message::Search(SearchMessage::PostsLoaded(posts))
                        }
                    }
                    Err(err) => {
                        error!("Couldn't load posts: {err}");
                        Message::Detail(DetailMessage::BackToGrid)
                    }
                },
            );
        }
        DetailMessage::BackToGrid => {
            app.selected_post = None;
        }
    }
    Task::none()
}

fn update_settings(app: &mut App, msg: SettingsMessage) -> Task<Message> {
    match msg {
        SettingsMessage::UsernameChanged(username) => {
            app.settings.username = username;
        }
        SettingsMessage::ApiKeyChanged(key) => {
            app.settings.username = key;
        }
        SettingsMessage::BlacklistEdited(action) => {
            app.settings.blacklist_content.perform(action);
        }
        SettingsMessage::Save => {
            debug!("Saving settings.");
            app.config.auth = Some(Auth {
                username: app.settings.username.clone(),
                api_key: app.settings.api_key.clone(),
            });

            let blacklist = app
                .settings
                .blacklist_content
                .text()
                .lines()
                .map(str::trim)
                .filter(|line| !line.is_empty())
                .map(str::to_string)
                .collect::<Vec<String>>();
            app.config.blacklist.rules = blacklist;

            app.config.followed_tags = app.followed.tags.clone();

            if let Err(err) = app.config.save() {
                warn!("Faled to save config: {err}");
            }

            app.ui.show_settings = false;
        }
    }
    Task::none()
}

fn update_followed(app: &mut App, msg: FollowedMessage) -> Task<Message> {
    match msg {
        FollowedMessage::CheckUpdates => {}
        FollowedMessage::UpdatesReceived(updates) => {}
        FollowedMessage::AddTag => {
            let tag = app.followed.new_followed_tag.trim();
            if !tag.is_empty() && !app.followed.tags.iter().any(|f| f.tag == tag) {
                app.followed.tags.push(FollowedTag {
                    tag: tag.to_string(),
                    last_seen_post_id: None,
                });

                app.config.followed_tags = app.followed.tags.clone();
                let _ = app.config.save();
            }
            app.followed.new_followed_tag.clear();
        }
        FollowedMessage::FollowTag(tag) => {}
        FollowedMessage::RemoveTag(tag) => {
            app.followed.tags.retain(|f| f.tag != tag);

            app.config.followed_tags = app.followed.tags.clone();
            let _ = app.config.save(); // TODO: make this a function of config?
        }
    }
    Task::none()
}

fn update_view(app: &mut App, msg: ViewMessage) -> Task<Message> {
    match msg {
        ViewMessage::ToggleSettings => {
            app.ui.show_settings = !app.ui.show_settings;
        }
        ViewMessage::WindowResized(width, height) => {
            app.ui.window_width = width;
            app.ui.window_height = height;
        }
    }
    Task::none()
}
