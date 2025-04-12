use super::model::Post;

pub fn is_blacklisted(post: &Post, rules: &[String]) -> bool {
    let tags = post.tags.iter().flat_map(|(_, t)| t).collect::<Vec<_>>();

    'rule: for rule in rules {
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
        return true;
    }

    return false;
}
