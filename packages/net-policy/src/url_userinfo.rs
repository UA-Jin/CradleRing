// Network Policy module implements url userinfo behavior.
// 翻译自 packages/net-policy/src/url-userinfo.ts

/// Strip username/password credentials from a URL string when it parses.
pub fn strip_url_user_info(value: &str) -> String {
    match url_parse(value) {
        Some(mut parsed) => {
            let has_userinfo = !parsed.username.is_empty() || !parsed.password.is_empty();
            if !has_userinfo {
                return value.to_string();
            }
            parsed.username.clear();
            parsed.password.clear();
            parsed.to_string()
        }
        None => value.to_string(),
    }
}

#[derive(Debug, Clone, Default)]
pub struct ParsedUrl {
    pub username: String,
    pub password: String,
    pub to_string: String,
}

impl std::fmt::Display for ParsedUrl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.to_string)
    }
}

pub fn url_parse(value: &str) -> Option<ParsedUrl> {
    match url::Url::parse(value) {
        Ok(u) => {
            let mut output = String::new();
            output.push_str(u.scheme());
            output.push_str("://");
            let user = u.username();
            let pass = u.password().unwrap_or("");
            if !user.is_empty() || !pass.is_empty() {
                if !user.is_empty() {
                    output.push_str(user);
                }
                if !pass.is_empty() {
                    output.push(':');
                    output.push_str(pass);
                }
                output.push('@');
            }
            if let Some(host) = u.host_str() {
                output.push_str(host);
            }
            if let Some(port) = u.port() {
                output.push(':');
                output.push_str(&port.to_string());
            }
            output.push_str(u.path());
            if let Some(query) = u.query() {
                output.push('?');
                output.push_str(query);
            }
            if let Some(fragment) = u.fragment() {
                output.push('#');
                output.push_str(fragment);
            }
            Some(ParsedUrl {
                username: user.to_string(),
                password: pass.to_string(),
                to_string: output,
            })
        }
        Err(_) => None,
    }
}