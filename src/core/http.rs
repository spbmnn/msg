use once_cell::sync::Lazy;
use reqwest::{Client, Method, RequestBuilder};

use super::config::Auth;

pub static CLIENT: Lazy<Client> = Lazy::new(|| {
    let user_agent = format!(
        "{}/{} (by homogoat on e621)",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    );
    Client::builder()
        .user_agent(user_agent)
        .build()
        .expect("failed to build reqwest Client")
});

pub fn authed_request(client: &Client, method: Method, url: &str, auth: &Auth) -> RequestBuilder {
    let request = client.request(method, url);
    request.basic_auth(&auth.username, Some(&auth.api_key))
}
