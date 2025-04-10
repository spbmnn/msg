//! Defines structs for API schemas.
//! Interpreted from the [e621 OpenAPI spec].
//!
//! [e621 OpenAPI spec]: https://e621.wiki/

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

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
            ("general", &self.general),
            ("artist", &self.artist),
            ("copyright", &self.copyright),
            ("character", &self.character),
            ("species", &self.species),
            ("invalid", &self.invalid),
            ("meta", &self.meta),
            ("lore", &self.lore),
        ]
        .into_iter()
    }
}

/*#[derive(Serialize, Deserialize, Debug)]
struct Relationships {
    parent_id: Option<u32>,
    has_children: bool,
    has_active_children: bool,
    children: Vec<u32>,
}*/

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
