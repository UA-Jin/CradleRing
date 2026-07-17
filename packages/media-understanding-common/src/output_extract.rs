// Output extractors for media-understanding provider CLI responses.
// 翻译自 packages/media-understanding-common/src/output-extract.ts

use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum QuoteChar {
    #[allow(dead_code)]
    None,
    Double,
    Single,
    Backtick,
}

impl QuoteChar {
    fn from_char(c: char) -> Option<Self> {
        match c {
            '"' => Some(QuoteChar::Double),
            '\'' => Some(QuoteChar::Single),
            '`' => Some(QuoteChar::Backtick),
            _ => None,
        }
    }
}

/** Parse the last JSON object in a noisy provider output string. */
fn extract_last_json_object(raw: &str) -> Option<Value> {
    let trimmed = raw.trim();
    let mut ranges: Vec<(usize, usize)> = Vec::new();
    let mut starts: Vec<usize> = Vec::new();
    let mut in_string = false;
    let mut escaped = false;
    let mut preamble_quote: Option<QuoteChar> = None;
    let mut preamble_escaped = false;
    let mut previous_significant: Option<char> = None;
    let mut line_has_non_whitespace = false;
    let mut array_depth: i32 = 0;
    let mut candidate_has_content = false;

    let chars: Vec<char> = trimmed.chars().collect();

    for index in 0..chars.len() {
        let character = chars[index];
        if in_string {
            if character == '\n' || character == '\r' {
                starts.clear();
                in_string = false;
                escaped = false;
            } else if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == '"' {
                in_string = false;
            }
            continue;
        }

        if starts.is_empty() {
            if let Some(pq) = preamble_quote {
                if character == '\n' || character == '\r' {
                    preamble_quote = None;
                    preamble_escaped = false;
                } else if preamble_escaped {
                    preamble_escaped = false;
                } else if character == '\\' {
                    preamble_escaped = true;
                } else {
                    let matching = match pq {
                        QuoteChar::Double => '"',
                        QuoteChar::Single => '\'',
                        QuoteChar::Backtick => '`',
                        QuoteChar::None => unreachable!(),
                    };
                    if character == matching {
                        preamble_quote = None;
                    }
                }
                continue;
            }
            if let Some(qc) = QuoteChar::from_char(character) {
                let previous = if index > 0 { chars.get(index - 1).copied() } else { None };
                let is_boundary = match previous {
                    None => true,
                    Some(p) => matches!(p, ' ' | '\t' | '\n' | '\r' | ':' | '(' | '[' | '{'),
                };
                if is_boundary {
                    preamble_quote = Some(qc);
                    preamble_escaped = false;
                    continue;
                }
            }
            if character == '{' {
                array_depth = 0;
                candidate_has_content = false;
                starts.push(index);
            }
            if !character.is_whitespace() {
                previous_significant = Some(character);
                line_has_non_whitespace = true;
            } else if character == '\n' || character == '\r' {
                line_has_non_whitespace = false;
            }
            continue;
        }

        let had_candidate_content = candidate_has_content;
        if character == '"' {
            in_string = true;
        } else if character == '{' {
            let prev = previous_significant;
            let valid_start = matches!(prev, Some(':') | Some('[') | Some('"'))
                || (prev == Some(',') && (line_has_non_whitespace || array_depth > 0));
            if valid_start {
                starts.push(index);
            } else if !line_has_non_whitespace && !had_candidate_content {
                // Only resync at a clean record boundary; otherwise keep malformed
                // outer objects from promoting diagnostic payloads as valid results.
                starts.truncate(1);
                if !starts.is_empty() {
                    starts[0] = index;
                } else {
                    starts.push(index);
                }
                array_depth = 0;
                candidate_has_content = false;
            }
        } else if character == '}' && !starts.is_empty() {
            let start = starts.pop();
            if let Some(s) = start {
                if starts.is_empty() {
                    ranges.push((s, index));
                }
            }
        } else if character == '[' {
            array_depth += 1;
        } else if character == ']' && array_depth > 0 {
            array_depth -= 1;
        }

        if !character.is_whitespace() {
            candidate_has_content = true;
            previous_significant = Some(character);
            line_has_non_whitespace = true;
        } else if character == '\n' || character == '\r' {
            line_has_non_whitespace = false;
        }
    }

    for range in ranges.iter().rev() {
        let slice: String = chars[range.0..=range.1].iter().collect();
        if let Ok(v) = serde_json::from_str::<Value>(&slice) {
            return Some(v);
        }
    }

    None
}

/** Extract Gemini CLI-style response text from the last JSON object in output. */
pub fn extract_gemini_response(raw: &str) -> Option<String> {
    let payload = extract_last_json_object(raw)?;
    if !payload.is_object() {
        return None;
    }
    let response = payload.get("response")?;
    if let Some(s) = response.as_str() {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    } else {
        None
    }
}