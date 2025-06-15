use super::model::Post;
use serde::{Deserialize, Serialize};
use tracing::debug;

#[derive(Debug, Deserialize, Default, Serialize, Clone, PartialEq)]
pub struct Blacklist {
    pub rules: Vec<String>,
}

pub fn is_blacklisted(post: &Post, blacklist: &Blacklist) -> bool {
    let tags = post.tags.iter().flat_map(|(_, t)| t).collect::<Vec<_>>();

    'rule: for rule in &blacklist.rules {
        let tokens = rule.split_whitespace();
        for token in tokens {
            if token.starts_with("-") {
                let neg = &token[1..];
                if tags.iter().any(|t| *t == neg) {
                    continue 'rule;
                }
            } else if !tags.iter().any(|t| *t == token) {
                continue 'rule;
            }
        }
        let post_id = post.id;
        debug!("Filtered post {post_id} for rule {rule}");
        return true;
    }

    return false;
}
