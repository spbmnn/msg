use reqwest::Method;
use std::cmp::{max, min};
use tracing::{debug, instrument, trace, Level};

use super::rate_limiter::API_LIMITER;
use super::{ApiError, BASE_URL};
use crate::config::Auth;
use crate::http::{authed_request, CLIENT};
use crate::model::Comment;

#[instrument(level = Level::TRACE)]
pub async fn fetch_comments(
    auth: Option<&Auth>,
    post_id: u32,
    page: Option<u32>,
) -> Result<Vec<Comment>, ApiError> {
    let mut page = match page {
        Some(p) => p,
        None => 1,
    };

    page = max(1, min(page, 750));

    let url = format!(
        "{BASE_URL}/comments.json?group_by=comment&search[post_id]={post_id}&search[order]=id_asc&page={page}"
    );

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

    let res: Vec<Comment> = serde_json::from_str(&text).unwrap_or_default();
    let length = &res.len();

    debug!("Got {length} comments for #{post_id}");
    Ok(res)
}
