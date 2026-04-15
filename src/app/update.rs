use crate::app::message::{
    DetailMessage, FollowedMessage, MediaMessage, Message, PostMessage, SearchMessage,
    SettingsMessage, ViewMessage,
};
use crate::app::state::{App, ViewMode};
use crate::core::api::{
    favorite_post, fetch_comments, fetch_posts, unfavorite_post, vote_post, FetchPoint,
};
use crate::core::config::Auth;
use crate::core::followed::compose_vec;
use crate::core::media::{fetch_gif, fetch_image, fetch_video};
use crate::core::media::{fetch_preview, fetch_sample};
use crate::core::model::{Post, PostType};
use crate::core::store::poststore_path;
use crate::core::{blacklist, followed, media};
use crate::gui::video_player::VideoPlayerWidget;
use iced::{clipboard, window, Task};
use tracing::{debug, error, info, instrument, trace, warn};

impl App {
    #[instrument(skip_all)]
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Search(msg) => self.update_search(msg),
            Message::Post(msg) => self.update_post(msg),
            Message::Media(msg) => self.update_media(msg),
            Message::Detail(msg) => self.update_detail(msg),
            Message::Settings(msg) => self.update_settings(msg),
            Message::Followed(msg) => self.update_followed(msg),
            Message::View(msg) => self.update_view(msg),
            Message::Tick => self.tick(),
            Message::Exit => self.exit(),
        }
    }

    fn update_search(&mut self, msg: SearchMessage) -> Task<Message> {
        match msg {
            SearchMessage::LoadPosts(query) => {
                self.loading = true;
                self.selected_post = None;
                self.ui.view_mode = ViewMode::Grid(query.clone(), self.search.page);
                if query != self.search.query {
                    self.posts.clear();
                }
                self.search.query = query.clone();
                self.search.input = query.clone();
                let auth = self.config.auth.clone();
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
                let fetch_point = match self.search.page {
                    Some(page_number) => {
                        self.search.page = Some(page_number + 1);
                        self.ui.view_mode =
                            ViewMode::Grid(self.search.query.clone(), self.search.page);
                        Some(FetchPoint::Page(page_number + 1))
                    }
                    None => {
                        self.search.page = Some(2);
                        self.ui.view_mode = ViewMode::Grid(self.search.query.clone(), Some(2));
                        Some(FetchPoint::Page(2))
                    }
                };
                self.loading = true;
                let query = self.search.input.clone();
                let auth = self.config.auth.clone();
                return Task::perform(
                    async move { fetch_posts(auth.as_ref(), query.clone(), fetch_point).await },
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
                self.loading = false;
                let filtered = posts
                    .into_iter()
                    .filter(|p| !blacklist::is_blacklisted(p, &self.config.blacklist))
                    .collect::<Vec<Post>>();
                self.store.insert_posts(filtered.clone());

                let mut post_ids: Vec<u32> = Vec::new();

                let mut queued_post_count: usize = 0;
                for post in &filtered {
                    post_ids.push(post.id);
                    if post.is_favorited {
                        self.store.set_favorite(post.id, true);
                    }
                    if post.preview.url.is_some() && !self.store.thumbnails.contains_key(&post.id) {
                        self.search.thumbnail_queue.push_back(post.id);
                        queued_post_count += 1;
                    }
                }

                self.store.update_results(&self.search.query, &post_ids);

                let new_posts = filtered
                    .into_iter()
                    .filter(|p| !self.posts.iter().any(|existing| existing.id == p.id))
                    .collect::<Vec<Post>>();
                self.posts.extend(new_posts.clone());

                info!("Loading thumbnails for {queued_post_count} posts");
            }
            SearchMessage::InputChanged(text) => {
                self.search.input = text;
            }
            SearchMessage::Submitted => {
                self.ui.history.proceed(self.ui.view_mode.clone());
                let query = self.search.input.trim().to_string();
                self.search.page = Some(1);
                self.search.query = query.clone();
                self.ui.view_mode = ViewMode::Grid(query.clone(), self.search.page);
                if !query.is_empty() {
                    info!("Submitting search for {query}");
                    let auth = self.config.auth.clone();
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
            SearchMessage::GetFavorites => {
                if self.config.auth.is_none() {
                    return Task::none();
                }
                self.posts.clear();
                let query = format!("fav:{}", self.config.auth.as_ref().unwrap().username);
                self.ui
                    .history
                    .proceed(ViewMode::Grid(query.clone(), self.search.page));
                return Task::done(Message::Search(SearchMessage::LoadPosts(query)));
            }
        }
        Task::none()
    }

    fn update_post(&mut self, msg: PostMessage) -> Task<Message> {
        match msg {
            PostMessage::View(id) => {
                self.ui.view_mode = ViewMode::Detail(id);
                self.selected_post = Some(id);
                info!("Selected post {id}");

                // Build task batch
                let mut commands = vec![];

                if self.store.is_favorited(id) {
                    self.store.get_post_mut(id).unwrap().is_favorited = true;
                }

                if let Some(post) = self.store.get_post(id) {
                    // TODO: Deal with .swfs for compatiblity.
                    // *Maaaaaaaybe* ruffle support? Doubt it.
                    match post.file.ext.as_deref() {
                        Some("gif") => {
                            if !self.store.has_gif(id) {
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
                            let url: String = if !self.store.has_video(id) {
                                post.file.url.clone().unwrap()
                            } else {
                                self.store.get_video(id).unwrap().to_string()
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
                            if !self.store.has_image(id) {
                                if self.config.view.download_sample {
                                    commands.push(Task::perform(
                                        fetch_sample(id, post.sample.clone()),
                                        move |res| match res {
                                            Ok(handle) => Message::Media(
                                                MediaMessage::SampleLoaded(id, handle),
                                            ),
                                            Err(err) => {
                                                error!("Sample {id} failed: {err}");
                                                Message::Tick
                                            }
                                        },
                                    ));
                                }
                                if self.config.view.download_fullsize {
                                    commands.push(Task::perform(
                                        fetch_image(id, post.file.clone()),
                                        move |res| match res {
                                            Ok(handle) => Message::Media(
                                                MediaMessage::ImageLoaded(id, handle),
                                            ),
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
                    commands.push(Task::perform(
                        fetch_comments(None, id, None),
                        move |res| match res {
                            Ok(comments) => {
                                Message::Detail(DetailMessage::CommentsLoaded(comments))
                            }
                            Err(err) => {
                                error!("Getting comments for {id} failed: {err}");
                                Message::Tick
                            }
                        },
                    ));
                }

                return Task::batch(commands);
            }
            PostMessage::Vote(id, vote) => {
                let auth = self.config.auth.clone().unwrap_or_default();
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
                let is_favorite = self.store.is_favorited(id);
                let auth = self.config.auth.clone().unwrap_or_default();
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
                    return Task::perform(
                        async move { favorite_post(&auth, id).await },
                        move |res| match res {
                            Ok(()) => Message::Post(PostMessage::FavoriteResult(id, true)),
                            Err(err) => {
                                error!("{err}");
                                Message::Tick
                            }
                        },
                    );
                }
            }
            PostMessage::FavoriteResult(id, favorited) => {
                self.store.set_favorite(id, favorited);
                Task::none()
            }
            PostMessage::VoteResult(id, result) => {
                self.store.set_vote(id, result);
                Task::none()
            }
        }
    }

    fn update_media(&mut self, msg: MediaMessage) -> Task<Message> {
        match msg {
            MediaMessage::ThumbnailLoaded(id, handle) => {
                debug!("Storing thumbnail for {id}");
                self.store.insert_thumbnail(id, handle);
            }
            MediaMessage::SampleLoaded(id, handle) => {
                debug!("Storing sample for {id}");
                self.store.insert_sample(id, handle);
            }
            MediaMessage::ImageLoaded(id, handle) => {
                debug!("Storing image for {id}");
                self.store.insert_image(id, handle);
            }
            MediaMessage::GifLoaded(id, gif) => {
                let frames = iced_gif::Frames::from_bytes(gif.clone());
                if let Ok(f) = frames {
                    debug!("Storing gif for {id}");
                    self.store.gif_frames.insert(id, f);
                } else {
                    error!("Couldn't decode gif into frames");
                }

                self.store.insert_gif(id, gif);
            }
            MediaMessage::VideoLoaded(id, url) => {
                debug!("Storing video for {id}");
                self.store.insert_video(id, url.clone());
                match media::build_video_pipeline(url.as_str()) {
                    Ok(video) => {
                        info!("Creating video player");
                        self.video_player = Some(VideoPlayerWidget::new(video));
                    }
                    Err(err) => {
                        error!("Failed to create video: {err}");
                        self.video_player = None;
                    }
                }
            }
            MediaMessage::VideoPlayerMsg(message) => {
                if let Some(player) = &mut self.video_player {
                    return player.update(message);
                }
            }
        }
        Task::none()
    }

    fn update_detail(&mut self, msg: DetailMessage) -> Task<Message> {
        match msg {
            DetailMessage::AddTagToSearch(tag) => {
                self.selected_post = None;
                self.search.query.push_str(&(" ".to_owned() + &tag));
                self.search.input = self.search.query.clone();
                self.loading = true;
                self.ui.view_mode = ViewMode::Grid(self.search.query.clone(), Some(1));
                let query = self.search.query.clone();

                return Task::done(Message::View(ViewMessage::Show(ViewMode::Grid(
                    query,
                    Some(1),
                ))));
            }
            DetailMessage::NegateTagFromSearch(tag) => {
                self.selected_post = None;
                self.search.query.push_str(&(" -".to_owned() + &tag));
                self.search.input = self.search.query.clone();
                self.loading = true;

                self.ui.view_mode = ViewMode::Grid(self.search.query.clone(), Some(1));
                let query = self.search.query.clone();

                return Task::done(Message::View(ViewMessage::Show(ViewMode::Grid(
                    query,
                    Some(1),
                ))));
            }
            DetailMessage::CommentsLoaded(comments) => {
                for comment in comments {
                    if let Some(comment_vec) = self.store.get_comments(comment.post_id) {
                        if comment_vec.contains(&comment) {
                            continue;
                        }
                    }
                    self.store.insert_comment(comment);
                }
            }
            DetailMessage::CopyURL => {
                if let Some(post) = self.selected_post {
                    let url = format!("https://e621.net/posts/{}", post);
                    info!("Copying {url} to clipboard");
                    return clipboard::write(url);
                }
            }
            DetailMessage::OpenFile => {
                if let Some(id) = self.selected_post {
                    if let Some(post) = self.store.get_post(id) {
                        match post.get_type() {
                            Some(PostType::Image) => {
                                if self.config.view.download_fullsize {
                                    let path = self.store.get_image_path(id);
                                    if path.exists() {
                                        let open_status = open::that_detached(path);
                                        match open_status {
                                            Ok(_) => {}
                                            Err(err) => {
                                                error!("Couldn't open file: {err}");
                                            }
                                        }
                                        return Task::none();
                                    }
                                }
                                if self.config.view.download_sample {
                                    let path = self.store.get_sample_path(id);
                                    if path.exists() {
                                        let open_status = open::that_detached(path);
                                        match open_status {
                                            Ok(_) => {}
                                            Err(err) => {
                                                error!("Couldn't open file: {err}");
                                            }
                                        }
                                        return Task::none();
                                    }
                                }
                            }
                            Some(PostType::Gif) => {
                                let path = self.store.get_gif_path(id);
                                if path.exists() {
                                    let open_status = open::that_detached(path);
                                    match open_status {
                                        Ok(_) => {}
                                        Err(err) => {
                                            error!("Couldn't open file: {err}");
                                        }
                                    }
                                    return Task::none();
                                }
                            }
                            Some(PostType::Video) => {
                                if let Some(url) = self.store.get_video(id) {
                                    if let Ok(path) = url.to_file_path() {
                                        if path.exists() {
                                            let open_status = open::that_detached(path);
                                            match open_status {
                                                Ok(_) => {}
                                                Err(err) => {
                                                    error!("Couldn't open file: {err}");
                                                }
                                            }
                                            return Task::none();
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        Task::none()
    }

    fn update_settings(&mut self, msg: SettingsMessage) -> Task<Message> {
        match msg {
            SettingsMessage::UsernameChanged(username) => {
                self.settings.username = username;
            }
            SettingsMessage::ApiKeyChanged(key) => {
                self.settings.api_key = key;
            }
            SettingsMessage::BlacklistEdited(action) => {
                self.settings.blacklist_content.perform(action);
            }
            SettingsMessage::FollowFieldChanged(field) => {
                self.followed.new_followed_tag = field;
            }
            SettingsMessage::SampleToggled(toggle) => {
                self.config.view.download_sample = toggle;
            }
            SettingsMessage::FullsizeToggled(toggle) => {
                self.config.view.download_fullsize = toggle;
            }
            SettingsMessage::Save => {
                debug!("Saving settings.");
                self.config.auth = Some(Auth {
                    username: self.settings.username.clone(),
                    api_key: self.settings.api_key.clone(),
                });

                let blacklist = self
                    .settings
                    .blacklist_content
                    .text()
                    .lines()
                    .map(str::trim)
                    .filter(|line| !line.is_empty())
                    .map(str::to_string)
                    .collect::<Vec<String>>();
                self.config.blacklist.rules = blacklist;

                self.config.followed_tags = compose_vec(self.followed.tags.clone());

                if let Err(err) = self.config.save() {
                    warn!("Failed to save config: {err}");
                }

                return Task::done(Message::View(ViewMessage::Show(
                    self.ui
                        .history
                        .previous(self.ui.view_mode.clone())
                        .unwrap_or(ViewMode::Grid(String::from("order:id_desc"), Some(1))),
                )));
            }
            SettingsMessage::PurgeCache => {
                let _purge_result = self.store.purge();
            }
            SettingsMessage::PPRChanged(ppr) => {
                self.config.view.posts_per_row = ppr;
            }
            SettingsMessage::TileSizeChanged(tile_size) => {
                self.config.view.tile_width = tile_size;
            }
        }
        Task::none()
    }

    fn update_followed(&mut self, msg: FollowedMessage) -> Task<Message> {
        match msg {
            FollowedMessage::CheckUpdates => {
                self.ui.history.proceed(self.ui.view_mode.clone());
                self.ui.view_mode = ViewMode::Followed;

                let tags = compose_vec(self.followed.tags.clone());
                let auth = self.config.auth.clone();

                return Task::perform(
                    async move { followed::check_for_updates(&tags, auth.as_ref()).await },
                    move |res| match res {
                        Ok(updates) => Message::Followed(FollowedMessage::UpdatesReceived(updates)),
                        Err(err) => {
                            error!("{err}");
                            Message::Tick
                        }
                    },
                );
            }
            FollowedMessage::UpdatesReceived(updates) => {
                for (_, posts) in &updates {
                    for post in posts {
                        if blacklist::is_blacklisted(post, &self.config.blacklist) {
                            continue;
                        }
                        let id = post.id;
                        self.store.insert_post(post.clone());
                        trace!("{post:?}");
                        if post.preview.url.is_some()
                            && !self.store.thumbnails.contains_key(&post.id)
                        {
                            trace!("Queueing thumbnail for {id}");
                            self.search.thumbnail_queue.push_back(post.id);
                        }
                    }
                }

                self.followed.new_followed_posts = updates;
            }
            FollowedMessage::AddTag => {
                let tag = self.followed.new_followed_tag.trim();
                if !tag.is_empty() && !self.followed.tags.keys().any(|f| f == tag) {
                    info!("Adding tag {tag}");
                    self.followed.tags.insert(tag.to_string(), None);

                    self.config.followed_tags = compose_vec(self.followed.tags.clone());
                    let _ = self.config.save();
                }
                self.followed.new_followed_tag.clear();
            }
            FollowedMessage::FollowTag(tag) => {
                self.followed.tags.insert(tag.to_string(), None);

                self.config.followed_tags = compose_vec(self.followed.tags.clone());
                let _ = self.config.save();
            }
            FollowedMessage::RemoveTag(tag) => {
                self.followed.tags.remove(&tag);

                self.config.followed_tags = compose_vec(self.followed.tags.clone());
                let _ = self.config.save();
            }
            FollowedMessage::ClearSeenPosts => {
                for (tag, posts) in self.followed.new_followed_posts.drain() {
                    if let Some(latest_post) = posts.first() {
                        if let Some(seen) = self.followed.tags.get_mut(&tag) {
                            *seen = Some(latest_post.id);
                        }
                    }
                }
            }
        }
        Task::none()
    }

    fn update_view(&mut self, msg: ViewMessage) -> Task<Message> {
        match msg {
            ViewMessage::Show(mode) => {
                self.ui.history.proceed(self.ui.view_mode.clone());
                match &mode {
                    ViewMode::Detail(id) => {
                        return Task::done(Message::Post(PostMessage::View(*id)))
                    }
                    ViewMode::Grid(query, page) => {
                        self.search.query = query.clone();
                        self.search.page = *page;
                        self.search.input = query.clone();
                        self.selected_post = None;
                        self.video_player = None;
                        self.ui.view_mode = mode.clone();
                        return Task::done(Message::Search(SearchMessage::LoadPosts(
                            query.clone(),
                        )));
                    }
                    _ => {}
                }
                self.selected_post = None;
                self.video_player = None;
                self.ui.view_mode = mode;

                debug!(?self.ui.history.backwards, ?self.ui.history.forwards);
            }
            ViewMessage::ShowWithoutProceed(mode) => {
                match &mode {
                    ViewMode::Detail(id) => {
                        return Task::done(Message::Post(PostMessage::View(*id)))
                    }
                    ViewMode::Grid(query, page) => {
                        self.search.query = query.clone();
                        self.search.input = query.clone();
                        self.search.page = *page;
                        self.ui.view_mode = mode.clone();
                        return Task::done(Message::Search(SearchMessage::LoadPosts(
                            query.clone(),
                        )));
                    }
                    _ => {}
                }
                self.selected_post = None;
                self.video_player = None;
                self.ui.view_mode = mode;
            }
            ViewMessage::WindowResized(width, height) => {
                self.ui.window_width = width;
                self.ui.window_height = height;
            }
            ViewMessage::Back => {
                if self.ui.history.backwards.is_empty() {
                    return Task::none();
                }
                self.video_player = None;
                match &self.ui.view_mode {
                    ViewMode::Settings => {
                        return Task::done(Message::Settings(SettingsMessage::Save))
                    }
                    ViewMode::Grid(query, page) => {
                        self.search.query = query.clone();
                        self.search.input = query.clone();
                        self.search.page = *page;
                    }
                    _ => {}
                }
                return Task::done(Message::View(ViewMessage::ShowWithoutProceed(
                    self.ui
                        .history
                        .previous(self.ui.view_mode.clone())
                        .unwrap_or(ViewMode::Grid(String::from("order:id_desc"), Some(1))),
                )));
            }
            ViewMessage::Forward => {
                if let Some(next_view) = self.ui.history.next(self.ui.view_mode.clone()) {
                    match &next_view {
                        ViewMode::Detail(id) => {
                            return Task::done(Message::Post(PostMessage::View(*id)))
                        }
                        ViewMode::Grid(query, page) => {
                            self.search.query = query.clone();
                            self.search.input = query.clone();
                            self.search.page = *page;
                        }
                        _ => {}
                    }
                    return Task::done(Message::View(ViewMessage::ShowWithoutProceed(
                        next_view.clone(),
                    )));
                }
            }
            ViewMessage::UpdateTheme(theme) => {
                self.config.view.theme = theme;
            }
        }
        Task::none()
    }

    fn tick(&mut self) -> Task<Message> {
        if let Some(post_id) = self.search.thumbnail_queue.pop_front() {
            if let Some(post) = self.store.get_post(post_id) {
                if let Some(url) = &post.preview.url {
                    return Task::perform(
                        fetch_preview(post_id, url.clone()),
                        move |res| match res {
                            Ok(thumb) => {
                                Message::Media(MediaMessage::ThumbnailLoaded(post_id, thumb))
                            }
                            Err(err) => {
                                error!("Failed to fetch thumbnail for {post_id}: {err}");
                                Message::Tick
                            }
                        },
                    );
                }
                warn!("Queue entry has no preview");
            }
        }
        Task::none()
    }

    fn exit(&mut self) -> Task<Message> {
        info!("exiting...");

        self.config.followed_tags = compose_vec(self.followed.tags.clone());

        match &self.config.save() {
            Ok(()) => info!("Saved config"),
            Err(err) => {
                error!("Couldn't save config: {err}");
            }
        }

        if let Some(path) = poststore_path() {
            match self.store.save_to(&path) {
                Ok(()) => info!("Saved PostStore to {path:?}"),
                Err(err) => {
                    error!("Couldn't save PostStore: {err}")
                }
            }
        } else {
            warn!("Couldn't find path for PostStore");
        }

        return window::latest().and_then(window::close);
    }
}
