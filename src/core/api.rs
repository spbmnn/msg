use reqwest::Method;
use serde::Deserialize;
use thiserror::Error;
use tracing::{debug, error, instrument, trace};

use super::config::Auth;
use super::http::{authed_request, CLIENT};
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

/// Index point for [`fetch_posts`]. Take the following snippet from the [e6 API wiki page](https://e621.net/wiki_pages/2425#posts_list):
/// > `page` The page that will be returned. Can also be used with `a` or `b` + post_id to get the posts after or before the specified post ID. For example `a13` gets every post after post_id 13 up to the limit. This overrides any ordering meta-tag, `order:id_desc` is always used instead.
#[derive(Debug, Clone, Copy)]
pub enum FetchPoint {
    Page(usize),
    Before(u32),
    After(u32),
}

impl FetchPoint {
    pub fn page_query(&self) -> String {
        match self {
            FetchPoint::Page(page_num) => format!("&page={page_num}"),
            FetchPoint::Before(post_id) => format!("&page=b{post_id}"),
            FetchPoint::After(post_id) => format!("&page=a{post_id}"),
        }
    }
}

#[instrument]
pub async fn fetch_posts(
    auth: Option<&Auth>,
    tag: String,
    fetch_point: Option<FetchPoint>,
) -> Result<Vec<Post>, ApiError> {
    let mut url = format!("{BASE_URL}/posts.json?tags={}", tag);

    if let Some(point) = fetch_point {
        url.push_str(&point.page_query());
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
    let posts = res.posts;
    let length = &posts.len();

    debug!("Got {length} results for {tag}");
    Ok(posts)
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

#[derive(Deserialize, Debug)]
struct FavoriteErrorResponse {
    message: String,
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
        if let Ok(error) = serde_json::from_str::<FavoriteErrorResponse>(&error_text) {
            if error.message == "You have already favorited this post" {
                return Ok(());
            }
        }
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
