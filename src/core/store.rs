use std::{
    collections::{HashMap, HashSet},
    fs,
    io::{BufReader, BufWriter},
    path::{Path, PathBuf},
    str::FromStr,
};

use directories::ProjectDirs;
use iced::widget::image::Handle;
use iced_gif::Frames;
use rmp_serde::Serializer;
use serde::{Deserialize, Serialize};
use serde_flow::Flow;
use thiserror::Error;
use tracing::{info, trace, warn};
use url::Url;

use super::{
    media::{gif_dir, image_dir, thumbnail_dir},
    model::{Comment, Post, Vote},
};

#[derive(Debug, Error)]
pub enum StoreError {
    #[error("IO error: {0}")]
    IOError(#[from] std::io::Error),

    #[error("RMP encoding error: {0}")]
    RmpEncodeError(#[from] rmp_serde::encode::Error),

    #[error("RMP decoding error: {0}")]
    RmpDecodeError(#[from] rmp_serde::decode::Error),
    //#[error("Voting error: {0}")]
    //VoteError(String),
}

/// Stores media for posts.
#[derive(Debug, Default)]
pub struct PostStore {
    /// List of [`Post`]s.
    pub posts: HashMap<u32, Post>,
    /// Post thumbnails, stored as [Handle]s of image data.
    pub thumbnails: HashMap<u32, Handle>,
    /// Post images, stored as [Handle]s of image data.
    pub images: HashMap<u32, Handle>,
    /// Post GIFs, stored as raw bytes.
    pub gifs: HashMap<u32, Vec<u8>>,
    /// Post GIFs, stored as [Frames].
    pub gif_frames: HashMap<u32, Frames>,
    /// Post videos, stored as the location on disk (as a [`Url`]).
    pub videos: HashMap<u32, Url>,

    /// Stored votes.
    /// Note that the e6 API has no way to see previous votes your account has made, so these are only votes made within MSG.
    pub votes: HashMap<u32, Vote>,
    /// List of posts (by ID) that have been favorited.
    pub favorites: HashSet<u32>,

    /// Stored comments. Currently not kept across sessions.
    pub comments: HashMap<u32, Vec<Comment>>,
}

/// Used for serializing [`PostStore`]s.
#[derive(Debug, Default, Serialize, Deserialize, Flow)]
#[flow(variant = 1)]
pub struct PostStoreData {
    /// List of [`Post`]s.
    pub posts: HashMap<u32, Post>,
    /// List of paths to thumbnail images stored on disk.
    pub thumbnails: HashMap<u32, PathBuf>,
    /// List of paths to post images stored on disk.
    pub images: HashMap<u32, PathBuf>,
    /// List of paths to GIFs stored on disk.
    pub gifs: HashMap<u32, PathBuf>,
    /// List of paths to videos stored on disk.
    /// TODO: Make this PathBuf like the rest.
    pub videos: HashMap<u32, String>,
    /// Stored votes.
    /// Note that the e6 API has no way to see previous votes your account has made, so these are only votes made within MSG.
    pub votes: HashMap<u32, bool>,
    /// List of posts (by ID) that have been favorited.
    pub favorites: HashSet<u32>,
}

impl PostStore {
    pub fn new() -> Self {
        Self {
            posts: HashMap::new(),
            votes: HashMap::new(),
            thumbnails: HashMap::new(),
            images: HashMap::new(),
            gifs: HashMap::new(),
            gif_frames: HashMap::new(),
            videos: HashMap::new(),
            favorites: HashSet::new(),
            comments: HashMap::new(),
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

    // --- Comments ---
    pub fn insert_comments(&mut self, id: u32, comments: impl IntoIterator<Item = Comment>) {
        if !self.comments.contains_key(&id) {
            self.comments.insert(id, Vec::new());
        }
        for comment in comments {
            self.comments
                .get_mut(&comment.post_id)
                .unwrap()
                .push(comment);
        }
    }

    pub fn get_comments(&self, id: u32) -> Option<&Vec<Comment>> {
        self.comments.get(&id)
    }

    // --- Votes ---

    pub fn set_vote(&mut self, post_id: u32, vote: Option<Vote>) {
        match vote {
            Some(v) => {
                self.votes.insert(post_id, v);
            }
            None => {
                self.votes.remove(&post_id);
            }
        }
    }

    /// Get the user's vote for a given post.
    pub fn vote_for(&self, post_id: u32) -> Option<Vote> {
        self.votes.get(&post_id).copied()
    }

    // --- Favorites ---

    pub fn is_favorited(&self, id: u32) -> bool {
        self.favorites.contains(&id)
    }

    pub fn set_favorite(&mut self, id: u32, favorited: bool) {
        if favorited {
            self.favorites.insert(id);
        } else {
            self.favorites.remove(&id);
        }
    }

    // --- Thumbnails ---

    pub fn insert_thumbnail(&mut self, id: u32, handle: Handle) {
        self.thumbnails.insert(id, handle);
    }

    pub fn get_thumbnail(&self, id: u32) -> Option<&Handle> {
        self.thumbnails.get(&id)
    }

    pub fn get_thumbnail_path(&self, id: u32) -> PathBuf {
        thumbnail_dir().join(format!("{id}.jpg"))
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

    pub fn get_image_path(&self, id: u32) -> PathBuf {
        image_dir().join(format!("{id}.png"))
    }

    // --- GIFs ---

    pub fn insert_gif(&mut self, id: u32, gif: Vec<u8>) {
        self.gifs.insert(id, gif);
    }

    pub fn get_gif(&self, id: u32) -> Option<&Vec<u8>> {
        self.gifs.get(&id)
    }

    pub fn get_gif_path(&self, id: u32) -> PathBuf {
        gif_dir().join(format!("{id}.gif"))
    }

    // --- Videos ---

    pub fn insert_video(&mut self, id: u32, url: Url) {
        self.videos.insert(id, url);
    }

    pub fn get_video(&self, id: u32) -> Option<&Url> {
        self.videos.get(&id)
    }

    // --- Utilities ---

    pub fn has_image(&self, id: u32) -> bool {
        self.images.contains_key(&id)
    }

    pub fn has_gif(&self, id: u32) -> bool {
        self.gifs.contains_key(&id)
    }

    pub fn has_video(&self, id: u32) -> bool {
        self.videos.contains_key(&id)
    }

    pub fn save_to(&self, path: &Path) -> Result<(), StoreError> {
        let data = PostStoreData {
            posts: self.posts.clone(),
            thumbnails: self
                .thumbnails
                .keys()
                .map(|&id| (id, self.get_thumbnail_path(id)))
                .collect(),
            images: self
                .images
                .keys()
                .map(|&id| (id, self.get_image_path(id)))
                .collect(),
            gifs: self
                .gifs
                .keys()
                .map(|&id| (id, self.get_gif_path(id)))
                .collect(),
            videos: self
                .videos
                .iter()
                .map(|(&id, url)| (id, url.to_string()))
                .collect(),
            votes: self
                .votes
                .iter()
                .map(|(&id, &vote)| (id, vote.into()))
                .collect(),
            favorites: self.favorites.clone(),
        };

        if !path.exists() {
            std::fs::create_dir_all(path.parent().unwrap())?;
        }

        let file = std::fs::File::create(path)?;
        let writer = BufWriter::new(file);
        data.serialize(&mut Serializer::new(writer))?;
        Ok(())
    }

    pub fn load_from(path: &Path) -> Result<Self, StoreError> {
        let file = std::fs::File::open(path)?;
        let reader = BufReader::new(file);
        let data: PostStoreData = rmp_serde::decode::from_read(reader)?;

        let mut store = PostStore::new();
        store.posts = data.posts;
        store.favorites = data.favorites;

        for (id, upvoted) in data.votes {
            store.set_vote(id, Some(Vote::from(upvoted)));
        }

        for (id, path) in data.thumbnails {
            if let Ok(bytes) = std::fs::read(path) {
                store.insert_thumbnail(id, Handle::from_bytes(bytes));
            }
        }

        for (id, path) in data.images {
            if let Ok(bytes) = std::fs::read(&path) {
                store.insert_image(id, Handle::from_bytes(bytes));
            }
        }

        for (id, path) in data.gifs {
            if let Ok(bytes) = std::fs::read(&path) {
                store.insert_gif(id, bytes);
            }
        }

        for (id, path) in data.videos {
            if let Ok(url) = Url::from_str(&path) {
                store.insert_video(id, url);
            }
        }

        let post_count = store.posts.len();
        info!("Loaded {post_count} posts");

        Ok(store)
    }

    /// Removes all non-favorited posts from storage.
    pub fn purge(&mut self) -> Result<usize, StoreError> {
        warn!("PURGE INITIATED!");

        let mut new_images = self.images.clone();
        let mut new_gifs = self.gifs.clone();
        let mut new_videos = self.videos.clone();

        // capture original lengths of all arrays
        let mut post_count = self.posts.len();
        let mut image_count = self.images.len();
        let mut gif_count = self.gifs.len();
        let mut video_count = self.videos.len();

        // to prevent borrow-checker shenanigans
        let favorites = self.favorites.clone();

        self.posts.retain(|id, _post| favorites.contains(id));
        post_count -= self.posts.len();
        info!("Removed {post_count} posts");

        for id in self.images.keys() {
            if !favorites.contains(&id) {
                let path = self.get_image_path(*id);
                fs::remove_file(&path)?;
                trace!("Removed {path:?}");
                new_images.remove(id);
            }
        }
        image_count -= new_images.len();
        info!("Removed {image_count} images");

        for id in self.gifs.keys() {
            if !favorites.contains(&id) {
                let path = self.get_gif_path(*id);
                fs::remove_file(&path)?;
                trace!("Removed {path:?}");
                new_gifs.remove(id);
                self.gif_frames.remove(id);
            }
        }
        gif_count -= new_gifs.len();
        info!("Removed {gif_count} gifs");

        for (id, url) in &self.videos {
            if !favorites.contains(&id) {
                let path = url.path();
                fs::remove_file(&path)?;
                trace!("Removed {path:?}");
                new_videos.remove(id);
            }
        }
        video_count -= new_videos.len();
        info!("Removed {video_count} videos");

        Ok(post_count)
    }
}

pub fn poststore_path() -> Option<PathBuf> {
    ProjectDirs::from("xyz", "stripywalrus", "msg")
        .map(|dirs| dirs.data_local_dir().join("store.mpk"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path_is_resolved() {
        let path = poststore_path().expect("should resolve");
        assert!(path.ends_with("store.mpk"));
    }
}
