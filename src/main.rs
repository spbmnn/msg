use anyhow::Result;
use api::{rate_limiter::RateLimiter, schema::Upload};
use base64::{engine::general_purpose, Engine as _};
use reqwest::{
    header::{self, HeaderValue, USER_AGENT},
    Response,
};
use std::{collections::HashMap, fs::File, io::Write};
use tracing::{debug, error, info};

mod api;

const MAX_DOWNLOADS: usize = 5;

#[allow(unused_variables)]
#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().init();
    debug!("loading .env");
    dotenvy::dotenv()?;
    let username = std::env::var("USERNAME")?;
    let api_key = std::env::var("API_KEY")?;
    let auth_secret: String = general_purpose::STANDARD.encode(format!("{}:{}", username, api_key));
    let mut auth_value: HeaderValue = HeaderValue::from_str(&auth_secret)?;
    auth_value.set_sensitive(true);

    let mut headers = header::HeaderMap::new();
    headers.insert(header::AUTHORIZATION, auth_value);

    let api_client_builder = reqwest::Client::builder()
        .user_agent("msg/0.1 (by homogoat on e621)")
        .default_headers(headers.clone());
    let api_client = api_client_builder.build()?;

    let rate_limiter = RateLimiter::new(tokio::time::Duration::from_secs(1), 1);

    let url = format!("https://e621.net/favorites.json?page=1");

    let response: Response = api_client
        .get(url)
        .header(USER_AGENT, "msg/0.1 (by homogoat on e621)")
        .basic_auth(username, Some(api_key))
        .send()
        .await?;

    let response_body = response.text().await?;

    let favs = &serde_json::from_str::<HashMap<&str, Vec<Upload>>>(&response_body)?["posts"];

    download_previews(favs, &api_client).await?;

    Ok(())
}

#[tracing::instrument(skip_all)]
async fn download_previews(posts: &Vec<Upload>, client: &reqwest::Client) -> Result<()> {
    let rate_limiter = RateLimiter::new(tokio::time::Duration::from_secs(1), MAX_DOWNLOADS);

    for post in posts {
        rate_limiter.acquire().await;
        let path: String = post.preview.url.clone();
        // TODO: check if file exists
        let mut download_to = File::create(format!("./cache/{}.jpg", post.id))?;
        info!("Downloading {} to ./cache/{}.jpg", path, post.id);
        download_to.write_all(&client.get(path).send().await?.bytes().await?)?;
    }
    Ok(())
}
