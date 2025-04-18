//! Defines structs for API schemas.
//! Interpreted from the [e621 OpenAPI spec].
//!
//! [e621 OpenAPI spec]: https://e621.wiki/

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};

/// Represents a post on e621.
#[derive(Clone, Serialize, Deserialize)]
pub struct Post {
    pub id: u32,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    pub file: File,
    pub preview: Preview,
    pub score: Score,
    pub tags: Tags,
    pub rating: Rating,
    //pub fav_count: u32,
    //pub sources: Vec<String>,
    //pub pools: Vec<u32>,
    //relationships: Relationships,
    pub description: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum Rating {
    #[serde(rename = "s")]
    Safe,
    #[serde(rename = "q")]
    Questionable,
    #[serde(rename = "e")]
    Explicit,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct File {
    pub ext: Option<String>,
    pub url: Option<String>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Preview {
    pub url: Option<String>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Score {
    pub total: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tags {
    pub general: Vec<String>,
    pub artist: Vec<String>,
    pub copyright: Vec<String>,
    pub character: Vec<String>,
    pub species: Vec<String>,
    pub invalid: Vec<String>,
    pub meta: Vec<String>,
    pub lore: Vec<String>,
}

impl Tags {
    pub fn iter(&self) -> impl Iterator<Item = (&'static str, &Vec<String>)> {
        [
            ("artist", &self.artist),
            ("copyright", &self.copyright),
            ("character", &self.character),
            ("species", &self.species),
            ("general", &self.general),
            ("invalid", &self.invalid),
            ("meta", &self.meta),
            ("lore", &self.lore),
        ]
        .into_iter()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FollowedTag {
    pub tag: String,
    #[serde(default)]
    pub last_seen_post_id: Option<u32>,
}

impl fmt::Display for FollowedTag {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.tag)
    }
}

impl fmt::Display for Post {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "post #{}", self.id)
    }
}

impl fmt::Debug for Post {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "post #{}", self.id)
    }
}

pub enum PostType {
    Image,
    Gif,
    Video,
    Flash,
}

impl Post {
    pub fn get_type(&self) -> Option<PostType> {
        match self.file.ext.as_deref() {
            Some("gif") => Some(PostType::Gif),
            Some("webm") | Some("mp4") => Some(PostType::Video),
            Some("swf") => Some(PostType::Flash),
            Some(_) => Some(PostType::Image),
            None => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Vote {
    Upvote = 1,
    Downvote = -1,
}
