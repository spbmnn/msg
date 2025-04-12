use serde::Deserialize;
use thiserror::Error;
use tracing::{debug, info, trace};

use crate::core::http::get_authed;

use super::config::Auth;
use super::http::CLIENT;
use super::model::Post;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Request failed: {0}")]
    Request(#[from] reqwest::Error),

    #[error("Deserialization failed: {0}")]
    Deserialize(#[from] serde_json::Error),
}

#[derive(Deserialize)]
struct ApiResponse {
    posts: Vec<Post>,
}

pub async fn fetch_posts(
    tag: String,
    before_id: Option<u32>,
    auth: Option<Auth>,
) -> Result<Vec<Post>, ApiError> {
    let mut url = format!("https://e621.net/posts.json?tags={}", tag);

    if let Some(id) = before_id {
        url.push_str(&format!("&page=b{}", id));
    }

    trace!("GET {url}");
    let res = get_authed(&CLIENT, &url, auth.as_ref())
        .send()
        .await?
        .json::<ApiResponse>()
        .await?;
    let length = &res.posts.len();

    info!("Got {length} results for {tag}");
    Ok(res.posts)
}
