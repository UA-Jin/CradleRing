// Markdown Core module implements frontmatter behavior.
// 翻译自 packages/markdown-core/src/frontmatter.ts
use once_cell::sync::Lazy;
use regex::Regex;
use serde_yaml::Value as YamlValue;
use std::collections::BTreeMap;

type ParsedFrontmatter = BTreeMap<String, String>;

#[derive(Debug, Clone)]
enum ParsedFrontmatterKind {
    Inline,
    Multiline,
}

#[derive(Debug, Clone)]
struct ParsedFrontmatterLineEntry {
    value: String,
    kind: ParsedFrontmatterKind,
    raw_inline: String,
}

#[derive(Debug, Clone)]
enum ParsedYamlKind {
    Scalar,
    Structured,
}

#[derive(Debug, Clone)]
struct ParsedYamlValue {
    value: String,
    kind: ParsedYamlKind,
}

fn strip_quotes(value: &str) -> String {
    if (value.starts_with('"') && value.ends_with('"') && value.len() >= 2)
        || (value.starts_with('\'') && value.ends_with('\'') && value.len() >= 2)
    {
        value[1..value.len() - 1].to_string()
    } else {
        value.to_string()
    }
}

fn coerce_yaml_frontmatter_value(value: &YamlValue) -> Option<ParsedYamlValue> {
    if value.is_null() {
        return None;
    }
    if let YamlValue::String(s) = value {
        return Some(ParsedYamlValue {
            value: s.trim().to_string(),
            kind: ParsedYamlKind::Scalar,
        });
    }
    if let YamlValue::Number(n) = value {
        return Some(ParsedYamlValue {
            value: n.to_string(),
            kind: ParsedYamlKind::Scalar,
        });
    }
    if let YamlValue::Bool(b) = value {
        return Some(ParsedYamlValue {
            value: b.to_string(),
            kind: ParsedYamlKind::Scalar,
        });
    }
    if value.is_mapping() || value.is_sequence() {
        match serde_json::to_string(value) {
            Ok(s) => Some(ParsedYamlValue {
                value: s,
                kind: ParsedYamlKind::Structured,
            }),
            Err(_) => None,
        }
    } else {
        None
    }
}

fn parse_yaml_frontmatter(block: &str) -> Option<BTreeMap<String, ParsedYamlValue>> {
    let parsed: serde_yaml::Result<YamlValue> = serde_yaml::from_str(block);
    let value = match parsed {
        Ok(v) => v,
        Err(_) => return None,
    };
    if !value.is_mapping() {
        return None;
    }
    let mapping = value.as_mapping()?;
    let mut result: BTreeMap<String, ParsedYamlValue> = BTreeMap::new();
    for (k, v) in mapping {
        let raw_key = k.as_str().unwrap_or("").to_string();
        let key = raw_key.trim().to_string();
        if key.is_empty() {
            continue;
        }
        if let Some(coerced) = coerce_yaml_frontmatter_value(v) {
            result.insert(key, coerced);
        }
    }
    Some(result)
}

fn extract_multi_line_value(lines: &[&str], start_index: usize) -> (String, usize) {
    let mut value_lines: Vec<&str> = Vec::new();
    let mut i = start_index + 1;

    while i < lines.len() {
        let line = match lines.get(i) {
            Some(l) => *l,
            None => break,
        };
        if !line.is_empty() && !line.starts_with(' ') && !line.starts_with('\t') {
            break;
        }
        value_lines.push(line);
        i += 1;
    }

    let combined = value_lines.join("\n").trim().to_string();
    (combined, i - start_index)
}

static LINE_FRONTMATTER_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^([\w-]+):\s*(.*)$").unwrap());

fn parse_line_frontmatter(block: &str) -> BTreeMap<String, ParsedFrontmatterLineEntry> {
    let mut result: BTreeMap<String, ParsedFrontmatterLineEntry> = BTreeMap::new();
    let lines: Vec<&str> = block.split('\n').collect();
    let mut i = 0usize;

    while i < lines.len() {
        let line = lines[i];
        let caps = match LINE_FRONTMATTER_RE.captures(line) {
            Some(c) => c,
            None => {
                i += 1;
                continue;
            }
        };
        let key = match caps.get(1) {
            Some(m) => m.as_str().to_string(),
            None => {
                i += 1;
                continue;
            }
        };
        let raw_inline_value = caps.get(2).map(|m| m.as_str()).unwrap_or("");
        let inline_value = raw_inline_value.trim();

        if inline_value.is_empty() && i + 1 < lines.len() {
            let next_line = lines.get(i + 1).copied().unwrap_or("");
            if next_line.starts_with(' ') || next_line.starts_with('\t') {
                let (value, lines_consumed) = extract_multi_line_value(&lines, i);
                if !value.is_empty() {
                    result.insert(
                        key.clone(),
                        ParsedFrontmatterLineEntry {
                            value,
                            kind: ParsedFrontmatterKind::Multiline,
                            raw_inline: inline_value.to_string(),
                        },
                    );
                }
                i += lines_consumed;
                continue;
            }
        }

        let value = strip_quotes(inline_value);
        if !value.is_empty() {
            result.insert(
                key,
                ParsedFrontmatterLineEntry {
                    value,
                    kind: ParsedFrontmatterKind::Inline,
                    raw_inline: inline_value.to_string(),
                },
            );
        }
        i += 1;
    }

    result
}

fn line_frontmatter_to_plain(
    parsed: &BTreeMap<String, ParsedFrontmatterLineEntry>,
) -> ParsedFrontmatter {
    let mut result: ParsedFrontmatter = BTreeMap::new();
    for (key, entry) in parsed {
        result.insert(key.clone(), entry.value.clone());
    }
    result
}

static YAML_BLOCK_SCALAR_INDICATOR_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[|>][+-]?(\d+)?[+-]?$").unwrap());

fn is_yaml_block_scalar_indicator(value: &str) -> bool {
    YAML_BLOCK_SCALAR_INDICATOR_RE.is_match(value)
}

fn should_prefer_inline_line_value(
    line_entry: &ParsedFrontmatterLineEntry,
    yaml_value: &ParsedYamlValue,
) -> bool {
    if !matches!(yaml_value.kind, ParsedYamlKind::Structured) {
        return false;
    }
    if !matches!(line_entry.kind, ParsedFrontmatterKind::Inline) {
        return false;
    }
    if is_yaml_block_scalar_indicator(&line_entry.raw_inline) {
        return false;
    }
    line_entry.value.contains(':')
}

#[derive(Debug, Clone)]
pub struct ExtractedFrontmatterBlock {
    pub block: String,
    pub body: String,
}

fn normalize_frontmatter_content(content: &str) -> String {
    let mut s = content.to_string();
    if s.starts_with('\u{FEFF}') {
        s = s[3..].to_string();
    }
    s = s.replace("\r\n", "\n").replace('\r', "\n");
    s
}

fn is_frontmatter_delimiter_line(line: &str) -> bool {
    line.trim_end() == "---"
}

fn extract_frontmatter_block_from_normalized(
    normalized: &str,
) -> Option<ExtractedFrontmatterBlock> {
    let first_line_end = normalized.find('\n');
    let first_line = match first_line_end {
        Some(idx) => &normalized[..idx],
        None => normalized,
    };
    if !is_frontmatter_delimiter_line(first_line) {
        return None;
    }
    if first_line_end.is_none() {
        return None;
    }
    let block_start = first_line_end.unwrap() + 1;

    let bytes = normalized.as_bytes();
    let len = bytes.len();
    let mut line_start = block_start;
    while line_start <= len {
        let mut line_end = len;
        for i in line_start..len {
            if bytes[i] == b'\n' {
                line_end = i;
                break;
            }
        }
        let current_line_end = line_end;
        let line = &normalized[line_start..current_line_end];
        if is_frontmatter_delimiter_line(line) {
            let block_end = if line_start > block_start {
                line_start - 1
            } else {
                line_start
            };
            let body_start = if line_end == len {
                len
            } else {
                line_end + 1
            };
            return Some(ExtractedFrontmatterBlock {
                block: normalized[block_start..block_end].to_string(),
                body: normalized[body_start..].to_string(),
            });
        }
        if line_end == len {
            break;
        }
        line_start = line_end + 1;
    }

    None
}

/// Splits a complete leading YAML frontmatter block from its Markdown body.
pub fn extract_frontmatter_block(content: &str) -> Option<ExtractedFrontmatterBlock> {
    let normalized = normalize_frontmatter_content(content);
    extract_frontmatter_block_from_normalized(&normalized)
}

/// Removes a leading YAML frontmatter block and returns the remaining Markdown body.
pub fn strip_frontmatter_block(content: &str) -> String {
    let normalized = normalize_frontmatter_content(content);
    extract_frontmatter_block_from_normalized(&normalized)
        .map(|b| b.body)
        .unwrap_or(normalized)
        .trim()
        .to_string()
}

/// Parses leading YAML frontmatter into string values used by skill and metadata loaders.
pub fn parse_frontmatter_block(content: &str) -> ParsedFrontmatter {
    let block = match extract_frontmatter_block(content) {
        Some(b) => b.block,
        None => return BTreeMap::new(),
    };

    let line_parsed = parse_line_frontmatter(&block);
    let yaml_parsed = parse_yaml_frontmatter(&block);
    let yaml_parsed = match yaml_parsed {
        Some(v) => v,
        None => return line_frontmatter_to_plain(&line_parsed),
    };

    let mut merged: ParsedFrontmatter = BTreeMap::new();
    for (key, yaml_value) in &yaml_parsed {
        merged.insert(key.clone(), yaml_value.value.clone());
        if let Some(line_entry) = line_parsed.get(key) {
            if should_prefer_inline_line_value(line_entry, yaml_value) {
                merged.insert(key.clone(), line_entry.value.clone());
            }
        }
    }

    for (key, line_entry) in &line_parsed {
        if !merged.contains_key(key) {
            merged.insert(key.clone(), line_entry.value.clone());
        }
    }

    merged
}