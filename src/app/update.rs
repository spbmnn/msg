use std::fs;

use crate::app::message::{
    DetailMessage, FollowedMessage, MediaMessage, Message, PostMessage, SearchMessage,
    SettingsMessage, StartupMessage, ViewMessage,
};
use crate::app::state::{App, ViewMode};
use crate::core::api::{favorite_post, fetch_posts, unfavorite_post, vote_post};
use crate::core::config::Auth;
use crate::core::media::fetch_preview;
use crate::core::media::{fetch_gif, fetch_image, fetch_video};
use crate::core::model::{FollowedTag, Post};
use crate::core::store::poststore_path;
use crate::core::{blacklist, followed, media};
use crate::gui::video_player::VideoPlayerWidget;
use iced::{window, Task};
use tracing::{debug, error, info, instrument, trace, warn};

#[instrument(skip_all)]
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
        Message::Tick => tick(app),
        Message::Exit => exit(app),
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
            app.ui.view_mode = ViewMode::Grid;
            app.search.query = query.clone();
            let auth = app.config.auth.clone();
            return Task::perform(
                async move { fetch_posts(auth.as_ref(), query.clone(), None).await },
                move |res| match res {
                    Ok(posts) => Message::Search(SearchMessage::PostsLoaded(posts)),
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
            let auth = app.config.auth.clone();
            return Task::perform(
                async move { fetch_posts(auth.as_ref(), query.clone(), before_id).await },
                move |res| match res {
                    Ok(posts) => Message::Search(SearchMessage::PostsLoaded(posts)),
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
                .filter(|p| !blacklist::is_blacklisted(p, &app.config.blacklist))
                .collect::<Vec<Post>>();
            app.store.insert_posts(filtered.clone());
            let new_posts = filtered
                .into_iter()
                .filter(|p| !app.posts.iter().any(|existing| existing.id == p.id))
                .collect::<Vec<Post>>();
            app.posts.extend(new_posts.clone());

            let mut queued_post_count: usize = 0;

            for post in &new_posts {
                if post.is_favorited {
                    app.store.set_favorite(post.id, true);
                }
                if post.preview.url.is_some() {
                    app.search.thumbnail_queue.push_back(post.id);
                    queued_post_count += 1;
                }
            }
            info!("Loading thumbnails for {queued_post_count} posts");
        }
        SearchMessage::InputChanged(text) => {
            app.search.input = text;
        }
        SearchMessage::Submitted => {
            app.posts.clear();
            let query = app.search.input.trim().to_string();
            app.search.query = query.clone();
            if !query.is_empty() {
                info!("Submitting search for {query}");
                let auth = app.config.auth.clone();
                return Task::perform(
                    async move { fetch_posts(auth.as_ref(), query.clone(), None).await },
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
            app.ui.view_mode = ViewMode::Detail(id);
            app.selected_post = Some(id);
            info!("Selected post {id}");

            // Build task batch
            let mut commands = vec![];

            if let Some(post) = app.store.get_post(id) {
                let file = &post.file.ext;
                // trace!("post file: {file:?}");
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
                        let url: String = if !app.store.has_video(id) {
                            post.file.url.clone().unwrap()
                        } else {
                            app.store.get_video(id).unwrap().to_string()
                        };
                        commands.push(Task::perform(
                            fetch_video(id, url, post.file.ext.clone().unwrap()),
                            move |res| match res {
                                Ok(url) => Message::Media(MediaMessage::VideoLoaded(id, url)),
                                Err(err) => {
                                    error!("Video {id} failed: {err}");
                                    Message::Tick
                                }
                            },
                        ));
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
        PostMessage::Vote(id, vote) => {
            let auth = app.config.auth.clone().unwrap_or_default();
            return Task::perform(
                async move { vote_post(&auth, id, vote).await },
                move |res| match res {
                    Ok(v) => Message::Post(PostMessage::VoteResult(id, v)),
                    Err(err) => {
                        error!("{err}");
                        Message::Tick
                    }
                },
            );
        }
        PostMessage::Favorite(id) => {
            let is_favorite = app.store.is_favorited(id);
            let auth = app.config.auth.clone().unwrap_or_default();
            if is_favorite {
                return Task::perform(
                    async move { unfavorite_post(&auth, id).await },
                    move |res| match res {
                        Ok(()) => Message::Post(PostMessage::FavoriteResult(id, false)),
                        Err(err) => {
                            error!("{err}");
                            Message::Tick
                        }
                    },
                );
            } else {
                return Task::perform(async move { favorite_post(&auth, id).await }, move |res| {
                    match res {
                        Ok(()) => Message::Post(PostMessage::FavoriteResult(id, true)),
                        Err(err) => {
                            error!("{err}");
                            Message::Tick
                        }
                    }
                });
            }
        }
        PostMessage::FavoriteResult(id, favorited) => {
            app.store.set_favorite(id, favorited);
            Task::none()
        }
        PostMessage::VoteResult(id, result) => {
            app.store.set_vote(id, result);
            Task::none()
        }
    }
}

fn update_media(app: &mut App, msg: MediaMessage) -> Task<Message> {
    match msg {
        MediaMessage::ThumbnailLoaded(id, handle) => {
            debug!("Storing thumbnail for {id}");
            app.store.insert_thumbnail(id, handle)
        }
        MediaMessage::ImageLoaded(id, handle) => {
            debug!("Storing image for {id}");
            app.store.insert_image(id, handle);
        }
        MediaMessage::GifLoaded(id, gif) => {
            let frames = iced_gif::Frames::from_bytes(gif.clone());
            if let Ok(f) = frames {
                debug!("Storing gif for {id}");
                app.store.gif_frames.insert(id, f);
            } else {
                error!("Couldn't decode gif into frames");
            }

            app.store.insert_gif(id, gif);
        }
        MediaMessage::VideoLoaded(id, url) => {
            debug!("Storing video for {id}");
            app.store.insert_video(id, url.clone());
            match media::build_video_pipeline(url.as_str()) {
                Ok(video) => {
                    info!("Creating video player");
                    app.video_player = Some(VideoPlayerWidget::new(video));
                }
                Err(err) => {
                    error!("Failed to create video: {err}");
                    app.video_player = None;
                }
            }
        }
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

            let auth = app.config.auth.clone();
            let query = app.search.query.clone();

            return Task::perform(
                async move { fetch_posts(auth.as_ref(), query, None).await },
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
                        Message::View(ViewMessage::ShowGrid)
                    }
                },
            );
        }
        DetailMessage::NegateTagFromSearch(tag) => {
            app.selected_post = None;
            app.search.query.push_str(&(" -".to_owned() + &tag));
            app.search.input = app.search.query.clone();
            app.loading = true;

            let auth = app.config.auth.clone();
            let query = app.search.query.clone();

            return Task::perform(
                async move { fetch_posts(auth.as_ref(), query, None).await },
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
                        Message::View(ViewMessage::ShowGrid)
                    }
                },
            );
        }
    }
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
        SettingsMessage::FollowFieldChanged(field) => {
            app.followed.new_followed_tag = field;
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

            app.ui.view_mode = ViewMode::Grid;
        }
    }
    Task::none()
}

fn update_followed(app: &mut App, msg: FollowedMessage) -> Task<Message> {
    match msg {
        FollowedMessage::CheckUpdates => {
            app.ui.view_mode = ViewMode::Followed;
            app.posts.clear();

            let tags = app.followed.tags.clone();
            let auth = app.config.auth.clone();

            return Task::perform(
                async move { followed::check_for_updates(tags, auth.as_ref()).await },
                move |res| match res {
                    Ok(updates) => Message::Followed(FollowedMessage::UpdatesReceived(updates)),
                    Err(err) => {
                        error!("{err}");
                        Message::View(ViewMessage::ShowGrid)
                    }
                },
            );
        }
        FollowedMessage::UpdatesReceived(updates) => {
            app.posts.clear();
            for (_, posts) in app.followed.new_followed_posts.clone() {
                for post in posts {
                    let id = post.id;
                    app.posts.push(post.clone());
                    app.store.insert_post(post.clone());
                    if post.preview.url.is_some() {
                        trace!("Queueing thumbnail for {id}");
                        app.search.thumbnail_queue.push_back(post.id);
                    }
                }
            }

            app.followed.new_followed_posts = updates;

            return Task::done(Message::Tick);
        }
        FollowedMessage::AddTag => {
            let tag = app.followed.new_followed_tag.trim();
            if !tag.is_empty() && !app.followed.tags.iter().any(|f| f.tag == tag) {
                info!("Adding tag {tag}");
                app.followed.tags.push(FollowedTag {
                    tag: tag.to_string(),
                    last_seen_post_id: None,
                });

                app.config.followed_tags = app.followed.tags.clone();
                let _ = app.config.save();
            }
            app.followed.new_followed_tag.clear();
        }
        FollowedMessage::FollowTag(tag) => {
            app.followed.tags.push(FollowedTag {
                tag: tag.to_string(),
                last_seen_post_id: None,
            });

            app.config.followed_tags = app.followed.tags.clone();
            let _ = app.config.save();
        }
        FollowedMessage::RemoveTag(tag) => {
            app.followed.tags.retain(|f| f.tag != tag);

            app.config.followed_tags = app.followed.tags.clone();
            let _ = app.config.save();
        }
    }
    Task::none()
}

fn update_view(app: &mut App, msg: ViewMessage) -> Task<Message> {
    match msg {
        ViewMessage::ShowSettings => {
            app.ui.view_mode = ViewMode::Settings;
        }
        ViewMessage::ShowGrid => {
            app.selected_post = None;
            app.ui.view_mode = ViewMode::Grid;
        }
        ViewMessage::WindowResized(width, height) => {
            app.ui.window_width = width;
            app.ui.window_height = height;
        }
    }
    Task::none()
}

fn tick(app: &mut App) -> Task<Message> {
    if let Some(post_id) = app.search.thumbnail_queue.pop_front() {
        if let Some(post) = app.posts.iter().find(|p| p.id == post_id) {
            if let Some(url) = &post.preview.url {
                let id = post.id;
                //trace!("Fetching thumbnail for {id}");
                return Task::perform(fetch_preview(post_id, url.clone()), move |res| match res {
                    Ok(thumb) => Message::Media(MediaMessage::ThumbnailLoaded(post_id, thumb)),
                    Err(err) => {
                        error!("Failed to fetch thumbnail for {post_id}: {err}");
                        Message::Tick
                    }
                });
            }
            warn!("Queue entry has no preview");
        }
    }
    Task::none()
}

fn exit(app: &mut App) -> Task<Message> {
    info!("exiting...");

    match &app.config.save() {
        Ok(()) => info!("Saved config"),
        Err(err) => {
            error!("Couldn't save config: {err}");
        }
    }

    if let Some(path) = poststore_path() {
        match app.store.save_to(&path) {
            Ok(()) => info!("Saved PostStore to {path:?}"),
            Err(err) => {
                error!("Couldn't save PostStore: {err}")
            }
        }
    } else {
        warn!("Couldn't find path for PostStore");
    }

    return window::get_latest().and_then(window::close);
}
