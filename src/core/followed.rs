use std::time::Duration;

use crate::core::api;
use crate::core::config::Auth;
use crate::core::model::{FollowedTag, Post};

use iced::futures::stream::{self, StreamExt};
use tokio::time::sleep;
use tracing::warn;

pub async fn check_for_updates(
    followed_tags: Vec<FollowedTag>,
    auth: Option<Auth>,
) -> Result<Vec<(String, Vec<Post>)>, api::ApiError> {
    const REQUEST_INTERVAL: Duration = Duration::from_millis(750);

    let mut updates = Vec::new();

    for tag in followed_tags {
        sleep(REQUEST_INTERVAL).await;

        let tag_name = tag.tag.clone();
        let last_seen = tag.last_seen_post_id;

        match api::fetch_posts(tag_name.clone(), None, auth.clone()).await {
            Ok(posts) => {
                let new_posts = match last_seen {
                    Some(id) => posts.into_iter().take_while(|p| p.id > id).collect(),
                    None => posts,
                };

                if !new_posts.is_empty() {
                    updates.push((tag_name, new_posts));
                }
            }
            Err(err) => {
                warn!("Failed to fetch for tag '{}': {err}", tag.tag);
            }
        }
    }

    Ok(updates)
}
