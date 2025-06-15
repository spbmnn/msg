use crate::api::{self, FetchPoint};
use crate::config::Auth;
use crate::model::Post;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use tracing::{instrument, warn};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct FollowedTag {
    pub tag: String,
    pub last_seen: Option<u32>,
}

#[instrument(skip(auth))]
pub async fn check_for_updates(
    followed_tags: &Vec<FollowedTag>,
    auth: Option<&Auth>,
) -> Result<FxHashMap<String, Vec<Post>>, api::ApiError> {
    let mut updates: FxHashMap<String, Vec<Post>> = FxHashMap::default();

    for tag in followed_tags {
        let fetch_point = match tag.last_seen {
            None => None,
            Some(id) => Some(FetchPoint::After(id)),
        };
        match api::fetch_posts(auth, tag.tag.clone(), fetch_point).await {
            Ok(posts) => {
                updates.insert(tag.tag.clone(), posts);
            }
            Err(err) => {
                warn!("Failed to fetch for tag '{}': {err}", tag.tag);
            }
        }
    }

    Ok(updates)
}

impl FollowedTag {
    pub fn decompose(self) -> (String, Option<u32>) {
        (self.tag, self.last_seen)
    }
}

pub fn compose_vec(map: FxHashMap<String, Option<u32>>) -> Vec<FollowedTag> {
    let mut tag_vec: Vec<FollowedTag> = Vec::new();
    for (tag, last_seen) in map.iter() {
        tag_vec.push(FollowedTag {
            tag: tag.clone(),
            last_seen: *last_seen,
        })
    }
    tag_vec
}

pub fn compose_hashmap(vec: Vec<FollowedTag>) -> FxHashMap<String, Option<u32>> {
    let mut tag_map: FxHashMap<String, Option<u32>> = FxHashMap::default();
    for ft in vec.iter() {
        tag_map.insert(ft.tag.clone(), ft.last_seen);
    }
    tag_map
}
