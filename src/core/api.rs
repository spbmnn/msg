use reqwest::Method;
use serde::Deserialize;
use thiserror::Error;
use tracing::{debug, instrument, trace};

use crate::core::http::authed_request;

use super::config::Auth;
use super::http::CLIENT;
use super::model::{Post, Vote};

pub mod comments;
pub mod rate_limiter;
use rate_limiter::API_LIMITER;

pub use comments::fetch_comments;

const BASE_URL: &str = "https://e621.net";

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Request failed: {0}")]
    Request(#[from] reqwest::Error),

    #[error("Deserialization failed: {0}")]
    Deserialize(#[from] serde_json::Error),

    #[error("Voting error: {0}")]
    VoteError(String),
}

#[derive(Deserialize)]
struct PostsResponse {
    posts: Vec<Post>,
}

#[instrument]
pub async fn fetch_posts(
    auth: Option<&Auth>,
    tag: String,
    before_id: Option<u32>,
) -> Result<Vec<Post>, ApiError> {
    let mut url = format!("{BASE_URL}/posts.json?tags={}", tag);

    if let Some(id) = before_id {
        url.push_str(&format!("&page=b{}", id));
    }

    trace!("GET {url}");
    let text: String = match auth {
        Some(auth) => {
            API_LIMITER
                .run(async {
                    authed_request(&CLIENT, Method::GET, &url, auth)
                        .send()
                        .await?
                        .text()
                        .await
                })
                .await?
        }
        None => {
            API_LIMITER
                .run(async { CLIENT.get(&url).send().await?.text().await })
                .await?
        }
    };
    //trace!("Raw response: {text}");
    let res: PostsResponse = serde_json::from_str(&text)?;
    let length = &res.posts.len();

    debug!("Got {length} results for {tag}");
    Ok(res.posts)
}

#[derive(Deserialize)]
struct VoteResponse {
    our_score: i8,
}

#[instrument(skip(auth))]
pub async fn vote_post(auth: &Auth, id: u32, vote: Option<Vote>) -> Result<Option<Vote>, ApiError> {
    let url = format!("{BASE_URL}/posts/{id}/votes.json");

    match vote {
        None => {
            trace!("DELETE {url}");
            let res = API_LIMITER
                .run(async {
                    authed_request(&CLIENT, Method::DELETE, &url, auth)
                        .json(&serde_json::json!({ "id": id }))
                        .send()
                        .await
                })
                .await?;

            if res.status().is_success() {
                return Ok(None);
            } else {
                let error_text = res.text().await.unwrap_or_default();
                return Err(ApiError::VoteError(error_text));
            }
        }
        Some(vote) => {
            trace!("POST {url}");
            let res = API_LIMITER
                .run(async {
                    authed_request(&CLIENT, Method::POST, &url, auth)
                        .json(&serde_json::json!({ "id": id, "score": vote as i8 }))
                        .send()
                        .await
                })
                .await?;

            if res.status().is_success() {
                let parsed: VoteResponse = res
                    .json()
                    .await
                    .map_err(|e| ApiError::VoteError(format!("Invalid vote response: {}", e)))?;

                let confirmed: Option<Vote> = match parsed.our_score {
                    1 => Some(Vote::Upvote),
                    -1 => Some(Vote::Downvote),
                    0 => None,
                    _ => {
                        return Err(ApiError::VoteError(format!(
                            "Vote failed: unexpected value {}",
                            parsed.our_score
                        )))
                    }
                };

                return Ok(confirmed);
            } else {
                let error_text = res.text().await.unwrap_or_default();
                return Err(ApiError::VoteError(error_text));
            }
        }
    }
}

#[instrument(skip(auth))]
pub async fn favorite_post(auth: &Auth, id: u32) -> Result<(), ApiError> {
    let url = format!("{BASE_URL}/favorites.json");

    trace!("POST {url}");
    let res = API_LIMITER
        .run(async {
            authed_request(&CLIENT, Method::POST, &url, auth)
                .json(&serde_json::json!({ "post_id": id }))
                .send()
                .await
        })
        .await?;

    if res.status().is_success() {
        return Ok(());
    } else {
        let error_text = res.text().await.unwrap_or_default();
        return Err(ApiError::VoteError(error_text));
    }
}

#[instrument(skip(auth))]
pub async fn unfavorite_post(auth: &Auth, id: u32) -> Result<(), ApiError> {
    let url = format!("{BASE_URL}/favorites/{id}.json");

    trace!("DELETE {url}");
    let res = API_LIMITER
        .run(async {
            authed_request(&CLIENT, Method::DELETE, &url, auth)
                .json(&serde_json::json!({ "post_id": id }))
                .send()
                .await
        })
        .await?;

    if res.status().is_success() {
        return Ok(());
    } else {
        let error_text = res.text().await.unwrap_or_default();
        return Err(ApiError::VoteError(error_text));
    }
}
