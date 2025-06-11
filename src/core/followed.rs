use crate::core::api::{self, FetchPoint};
use crate::core::config::Auth;
use crate::core::model::Post;
use rustc_hash::FxHashMap;
use tracing::{instrument, warn};

#[instrument(skip(auth))]
pub async fn check_for_updates(
    followed_tags: FxHashMap<String, Option<u32>>,
    auth: Option<&Auth>,
) -> Result<FxHashMap<String, Vec<Post>>, api::ApiError> {
    let mut updates: FxHashMap<String, Vec<Post>> = FxHashMap::default();

    for (tag, last_seen) in followed_tags {
        let fetch_point = match last_seen {
            None => None,
            Some(id) => Some(FetchPoint::After(id)),
        };
        match api::fetch_posts(auth, tag.clone(), fetch_point).await {
            Ok(posts) => {
                updates.insert(tag, posts);
            }
            Err(err) => {
                warn!("Failed to fetch for tag '{}': {err}", tag);
            }
        }
    }

    Ok(updates)
}
