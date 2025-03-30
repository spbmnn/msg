use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt};

#[derive(Serialize, Deserialize, Debug)]
pub enum Rating {
    #[serde(rename = "s")]
    Safe,
    #[serde(rename = "q")]
    Questionable,
    #[serde(rename = "e")]
    Explicit,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct File {
    width: u32,
    height: u32,
    pub ext: String,
    size: u32,
    pub md5: String,
    pub url: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Preview {
    width: u32,
    height: u32,
    pub url: String,
}

type Sample = Option<Preview>;

#[derive(Serialize, Deserialize, Debug)]
pub struct Score {
    up: i32,
    down: i32,
}

#[derive(Serialize, Deserialize, Debug)]
struct Relationships {
    parent_id: Option<u32>,
    has_children: bool,
    has_active_children: bool,
    children: Vec<u32>,
}

#[derive(Serialize, Deserialize)]
pub struct Upload {
    pub id: u32,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    pub file: File,
    pub preview: Preview,
    sample: Sample,
    pub score: Score,
    pub tags: HashMap<String, Vec<String>>,
    pub rating: Rating,
    pub fav_count: u32,
    pub sources: Vec<String>,
    pub pools: Vec<u32>,
    relationships: Relationships,
    pub description: String,
}

impl fmt::Display for Upload {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "post #{}", self.id)
    }
}

impl fmt::Debug for Upload {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "post #{}", self.id)
    }
}
