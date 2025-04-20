use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::{BufReader, BufWriter, Read, Write},
    path::{Path, PathBuf},
};

use directories::ProjectDirs;
use iced::widget::image::Handle;
use iced_gif::Frames;
use rmp_serde::Serializer;
use serde::Serialize;
use thiserror::Error;
use tracing::{debug, trace};
use url::Url;

use super::model::{Post, Vote};

#[derive(Debug, Error)]
pub enum StoreError {
    #[error("IO error: {0}")]
    IOError(#[from] std::io::Error),

    #[error("RMP encoding error: {0}")]
    RmpEncodeError(#[from] rmp_serde::encode::Error),

    #[error("RMP decoding error: {0}")]
    RmpDecodeError(#[from] rmp_serde::decode::Error),

    #[error("Voting error: {0}")]
    VoteError(String),
}

/// Stores media for posts.
#[derive(Debug, Default)]
pub struct PostStore {
    posts: HashMap<u32, Post>,
    thumbnails: HashMap<u32, Handle>,
    images: HashMap<u32, Handle>,
    gifs: HashMap<u32, Vec<u8>>,
    pub gif_frames: HashMap<u32, Frames>,
    videos: HashMap<u32, Url>,

    votes: HashMap<u32, Vote>,
    favorites: HashSet<u32>,
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

    pub fn save_votes_to(&self, path: &Path) -> Result<(), StoreError> {
        let file = File::create(path)?;
        let writer = BufWriter::new(file);

        Ok(self.votes.serialize(&mut Serializer::new(writer))?)
    }

    pub fn load_votes_from(&mut self, path: &Path) -> Result<(), StoreError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        self.votes = rmp_serde::decode::from_read(reader)?;
        Ok(())
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

pub fn data_dir() -> PathBuf {
    ProjectDirs::from("xyz", "stripywalrus", "msg")
        .unwrap()
        .data_dir()
        .to_path_buf()
}
