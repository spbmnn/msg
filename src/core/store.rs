use std::{collections::HashMap, path::PathBuf};

use directories::ProjectDirs;
use iced::widget::image::Handle;
use iced_gif::Frames;
use iced_video_player::Video;
use tracing::{debug, trace};
use url::Url;

use super::model::Post;

/// Stores media for posts.
#[derive(Debug, Default)]
pub struct PostStore {
    posts: HashMap<u32, Post>,
    thumbnails: HashMap<u32, Handle>,
    images: HashMap<u32, Handle>,
    gifs: HashMap<u32, Vec<u8>>,
    videos: HashMap<u32, Url>,
}

impl PostStore {
    pub fn new() -> Self {
        Self {
            posts: HashMap::new(),
            thumbnails: HashMap::new(),
            images: HashMap::new(),
            gifs: HashMap::new(),
            videos: HashMap::new(),
        }
    }

    // --- Posts ---

    pub fn insert_post(&mut self, post: Post) {
        self.posts.insert(post.id, post);
    }

    pub fn insert_posts(&mut self, posts: impl IntoIterator<Item = Post>) {
        for post in posts {
            self.posts.insert(post.id, post);
        }
    }

    pub fn get_post(&self, id: u32) -> Option<&Post> {
        self.posts.get(&id)
    }

    pub fn all_posts(&self) -> impl Iterator<Item = &Post> {
        self.posts.values()
    }

    // --- Thumbnails ---

    pub fn insert_thumbnail(&mut self, id: u32, handle: Handle) {
        self.thumbnails.insert(id, handle);
    }

    pub fn get_thumbnail(&self, id: u32) -> Option<&Handle> {
        self.thumbnails.get(&id)
    }

    // --- Images ---

    pub fn insert_image(&mut self, id: u32, handle: Handle) {
        trace!(post_id = id, "Inserting image into store");
        self.images.insert(id, handle);
    }

    pub fn get_image(&self, id: u32) -> Option<&Handle> {
        let result = self.images.get(&id);
        trace!(
            post_id = id,
            found = result.is_some(),
            "get_image: found? {}",
            result.is_some()
        );

        result
    }

    // --- GIFs ---

    pub fn insert_gif(&mut self, id: u32, gif: Vec<u8>) {
        self.gifs.insert(id, gif);
    }

    pub fn get_gif(&self, id: u32) -> Option<&Vec<u8>> {
        self.gifs.get(&id)
    }

    // --- Videos ---

    pub fn insert_video(&mut self, id: u32, url: Url) {
        self.videos.insert(id, url);
    }

    pub fn get_video(&self, id: u32) -> Option<&Url> {
        self.videos.get(&id)
    }

    // --- Utilities ---

    pub fn clear(&mut self) {
        self.posts.clear();
        self.thumbnails.clear();
        self.images.clear();
        self.gifs.clear();
        self.videos.clear();
    }

    pub fn has_image(&self, id: u32) -> bool {
        self.images.contains_key(&id)
    }

    pub fn has_gif(&self, id: u32) -> bool {
        self.gifs.contains_key(&id)
    }

    pub fn has_video(&self, id: u32) -> bool {
        self.videos.contains_key(&id)
    }
}
