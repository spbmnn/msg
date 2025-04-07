use serde::Deserialize;
use thiserror::Error;
use tracing::debug;

use super::http::CLIENT;
use super::model::Post;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Request failed: {0}")]
    Request(#[from] reqwest::Error),

    #[error("Deserialization failed: {0}")]
    Deserialize(#[from] serde_json::Error),

    #[error("Missing 'posts' field in API response")]
    MissingPosts,
}

#[derive(Deserialize)]
struct ApiResponse {
    posts: Vec<Post>,
}

pub async fn fetch_posts(tag: String, page: u32) -> Result<Vec<Post>, ApiError> {
    debug!("Fetching page {page} of {tag}");
    let url = format!("https://e621.net/posts.json?tags={}&page={}", tag, page);

    let res = CLIENT.get(url).send().await?.json::<ApiResponse>().await?;
    let length = &res.posts.len();

    debug!("Got {length} results for {tag}");
    Ok(res.posts)
}
