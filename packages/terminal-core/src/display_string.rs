// Terminal Core module implements display string behavior.
// 翻译自 packages/terminal-core/src/display-string.ts

use std::env;
use std::path;

use once_cell::sync::Lazy;
use regex::Regex;

pub type EnvMap = std::collections::HashMap<String, String>;

fn normalize(value: Option<&str>) -> Option<String> {
    let trimmed = value.map(|v| v.trim()).unwrap_or("");
    if trimmed.is_empty() || trimmed == "undefined" || trimmed == "null" {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn normalize_safe<F: FnOnce() -> Option<String>>(fn_: F) -> Option<String> {
    match fn_() {
        Some(s) => normalize(Some(&s)),
        None => None,
    }
}

fn resolve_termux_home(env: &EnvMap) -> Option<String> {
    let prefix = normalize(env.get("PREFIX").map(|s| s.as_str()))?;
    if normalize(env.get("ANDROID_DATA").map(|s| s.as_str())).is_none() {
        return None;
    }
    let prefix_unix = prefix.replace('\\', "/");
    let re = Regex::new(r"(?:^|/)(?:com\.termux/files/usr/?)$").ok()?;
    if !re.is_match(&prefix_unix) {
        return None;
    }
    let parent = path::Path::new(&prefix).parent()?;
    let home = parent.join("home");
    Some(home.to_string_lossy().to_string())
}

fn os_homedir() -> Option<String> {
    env::var_os("HOME").map(|s| s.to_string_lossy().to_string())
}

fn resolve_raw_os_home_dir(env: &EnvMap, homedir: fn() -> Option<String>) -> Option<String> {
    normalize(env.get("HOME").map(|s| s.as_str()))
        .or_else(|| normalize(env.get("USERPROFILE").map(|s| s.as_str())))
        .or_else(|| resolve_termux_home(env))
        .or_else(|| normalize_safe(|| homedir()))
}

fn resolve_raw_home_dir(env: &EnvMap, homedir: fn() -> Option<String>) -> Option<String> {
    let explicit_home = normalize(env.get("OPENCLAW_HOME").map(|s| s.as_str()));
    if let Some(explicit) = explicit_home {
        let fallback = resolve_raw_os_home_dir(env, homedir);
        let resolved = if let Some(fb) = &fallback {
            if explicit.starts_with('~') {
                let chars: Vec<char> = explicit.chars().collect();
                let next_char = chars.get(1).copied().unwrap_or('\0');
                if next_char == '\0' || next_char == '/' || next_char == '\\' {
                    format!("{}{}", fb, &explicit[1..])
                } else {
                    explicit.clone()
                }
            } else {
                explicit.clone()
            }
        } else {
            explicit.clone()
        };
        return Some(resolved);
    }
    resolve_raw_os_home_dir(env, homedir)
}

fn resolve_effective_home_dir(env: &EnvMap, homedir: fn() -> Option<String>) -> Option<String> {
    let raw = resolve_raw_home_dir(env, homedir)?;
    Some(path::Path::new(&raw).canonicalize().ok()?.to_string_lossy().to_string())
}

struct HomeDisplay {
    home: String,
    prefix: String,
}

fn resolve_home_display_prefix() -> Option<HomeDisplay> {
    let env: EnvMap = env::vars().collect();
    let home = resolve_effective_home_dir(&env, os_homedir)?;
    let explicit = env::var("OPENCLAW_HOME")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    match explicit {
        Some(_) => Some(HomeDisplay {
            home,
            prefix: "$OPENCLAW_HOME".to_string(),
        }),
        None => Some(HomeDisplay {
            home,
            prefix: "~".to_string(),
        }),
    }
}

static TOKEN_START_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r#"[\s("'`:=\[,{]"#).unwrap());
static PUNCT_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r#"[)"'`:,;.\]}]"#).unwrap());

fn replace_home_path(input: &str, display: &HomeDisplay) -> String {
    let mut output = String::new();
    let mut cursor: usize = 0;
    let home = &display.home;
    let prefix = &display.prefix;

    while cursor < input.len() {
        let index = match input[cursor..].find(home) {
            Some(offset) => cursor + offset,
            None => {
                output.push_str(&input[cursor..]);
                return output;
            }
        };

        let before = if index > 0 {
            input[..index].chars().last()
        } else {
            None
        };
        let home_end = index + home.len();
        let after = input[home_end..].chars().next();
        let starts_token = match before {
            None => true,
            Some(b) => {
                let s = b.to_string();
                TOKEN_START_RE.is_match(&s)
            }
        };

        let mut punctuation_end = home_end;
        let bytes = input.as_bytes();
        while punctuation_end < input.len() {
            let ch = bytes[punctuation_end] as char;
            if PUNCT_RE.is_match(&ch.to_string()) {
                punctuation_end += 1;
            } else {
                break;
            }
        }
        let punctuation_ends_token = punctuation_end > home_end
            && (punctuation_end == input.len()
                || {
                    let ch = bytes[punctuation_end] as char;
                    ch.is_whitespace()
                });
        let ends_token = match after {
            None => true,
            Some(c) => c == '/' || c == '\\' || punctuation_ends_token,
        };
        if starts_token && ends_token {
            output.push_str(&input[cursor..index]);
            output.push_str(prefix);
        } else {
            output.push_str(&input[cursor..index + home.len()]);
        }
        cursor = index + home.len();
    }

    output
}

/// Replace the effective home path with "~" or "$OPENCLAW_HOME" for terminal display.
pub fn display_string(input: &str) -> String {
    if input.is_empty() {
        return input.to_string();
    }
    match resolve_home_display_prefix() {
        Some(display) => replace_home_path(input, &display),
        None => input.to_string(),
    }
}
