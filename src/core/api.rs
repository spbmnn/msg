use reqwest::Method;
use serde::Deserialize;
use thiserror::Error;
use tracing::{debug, info, trace};

use crate::core::http::authed_request;

use super::config::Auth;
use super::http::CLIENT;
use super::model::{Post, Vote};

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

pub async fn fetch_posts(
    tag: String,
    before_id: Option<u32>,
    auth: Option<Auth>,
) -> Result<Vec<Post>, ApiError> {
    let mut url = format!("{BASE_URL}/posts.json?tags={}", tag);

    if let Some(id) = before_id {
        url.push_str(&format!("&page=b{}", id));
    }

    trace!("GET {url}");
    let res = authed_request(&CLIENT, Method::GET, &url, auth.as_ref())
        .send()
        .await?
        .json::<PostsResponse>()
        .await?;
    let length = &res.posts.len();

    info!("Got {length} results for {tag}");
    Ok(res.posts)
}

#[derive(Deserialize)]
struct VoteResponse {
    our_score: i8,
}

pub async fn vote_post(id: u32, vote: Option<Vote>, auth: Auth) -> Result<Option<Vote>, ApiError> {
    let url = format!("{BASE_URL}/posts/{id}/votes.json");

    match vote {
        None => {
            let res = authed_request(&CLIENT, Method::DELETE, &url, Some(&auth))
                .json(&serde_json::json!({ "id": id }))
                .send()
                .await?;

            if res.status().is_success() {
                return Ok(None);
            } else {
                let error_text = res.text().await.unwrap_or_default();
                return Err(ApiError::VoteError(error_text));
            }
        }
        Some(vote) => {
            let res = authed_request(&CLIENT, Method::POST, &url, Some(&auth))
                .json(&serde_json::json!({ "id": id, "score": vote as i8 }))
                .send()
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

pub async fn favorite_post(id: u32, auth: Auth) -> Result<(), ApiError> {
    let url = format!("{BASE_URL}/favorites.json");

    let res = authed_request(&CLIENT, Method::POST, &url, Some(&auth))
        .json(&serde_json::json!({ "post_id": id }))
        .send()
        .await?;

    if res.status().is_success() {
        return Ok(());
    } else {
        let error_text = res.text().await.unwrap_or_default();
        return Err(ApiError::VoteError(error_text));
    }
}

pub async fn unfavorite_post(id: u32, auth: Auth) -> Result<(), ApiError> {
    let url = format!("{BASE_URL}/favorites.json");

    let res = authed_request(&CLIENT, Method::DELETE, &url, Some(&auth))
        .json(&serde_json::json!({ "post_id": id }))
        .send()
        .await?;

    if res.status().is_success() {
        return Ok(());
    } else {
        let error_text = res.text().await.unwrap_or_default();
        return Err(ApiError::VoteError(error_text));
    }
}
