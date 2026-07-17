//! Robust JSON parser for partial / streaming responses.
//! 翻译自 packages/ai/src/utils/json-parse.ts
//!
//! Streams of tokens from an LLM may produce incomplete JSON strings
//! (especially for tool arguments). The TS helper used the `partial-json`
//! library to parse the longest valid prefix; in Rust we implement an
//! equivalent brace/bracket tracker that extracts balanced JSON values
//! from the input buffer.

use serde_json::Value;

/// Extracts all complete top-level JSON values from `buffer`.
/// Returns the parsed values plus the remaining (incomplete) tail.
pub fn extract_complete_json(buffer: &str) -> (Vec<Value>, String) {
    let mut values: Vec<Value> = Vec::new();
    let mut start: Option<usize> = None;
    let mut depth: i64 = 0;
    let mut in_string = false;
    let mut escape = false;
    let bytes = buffer.as_bytes();
    let mut last_complete_end: Option<usize> = None;

    for (i, &b) in bytes.iter().enumerate() {
        let c = b as char;
        if escape {
            escape = false;
            continue;
        }
        if in_string {
            if c == '\\' {
                escape = true;
            } else if c == '"' {
                in_string = false;
            }
            continue;
        }
        match c {
            '"' => in_string = true,
            '{' | '[' => {
                if depth == 0 {
                    start = Some(i);
                }
                depth += 1;
            }
            '}' | ']' => {
                if depth > 0 {
                    depth -= 1;
                    if depth == 0 {
                        if let Some(s) = start {
                            let slice = &buffer[s..=i];
                            if let Ok(v) = serde_json::from_str::<Value>(slice) {
                                values.push(v);
                                last_complete_end = Some(i + 1);
                            }
                        }
                        start = None;
                    }
                }
            }
            _ => {}
        }
    }

    let remainder = match last_complete_end {
        Some(end) => buffer[end..].to_string(),
        None => buffer.to_string(),
    };
    (values, remainder)
}

/// Try to parse a single JSON value. Returns `None` if the buffer is incomplete.
pub fn try_parse(buffer: &str) -> Option<Value> {
    serde_json::from_str::<Value>(buffer.trim()).ok()
}

/// Try to parse a single JSON value, recovering from trailing commas
/// and missing closing braces/brackets by adding them heuristically.
pub fn try_parse_lenient(buffer: &str) -> Option<Value> {
    let trimmed = buffer.trim();
    if trimmed.is_empty() {
        return None;
    }
    if let Ok(v) = serde_json::from_str::<Value>(trimmed) {
        return Some(v);
    }
    // Heuristic: count unbalanced braces/brackets, append closing ones.
    let mut opens: Vec<char> = Vec::new();
    let mut in_string = false;
    let mut escape = false;
    for c in trimmed.chars() {
        if escape {
            escape = false;
            continue;
        }
        if in_string {
            if c == '\\' {
                escape = true;
            } else if c == '"' {
                in_string = false;
            }
            continue;
        }
        match c {
            '"' => in_string = true,
            '{' | '[' => opens.push(c),
            '}' | ']' => {
                if let Some(last) = opens.last() {
                    let expected = if *last == '{' { '}' } else { ']' };
                    if c == expected {
                        opens.pop();
                    }
                }
            }
            _ => {}
        }
    }
    if opens.is_empty() && !in_string {
        return None;
    }
    let mut repaired = trimmed.trim_end_matches(',').to_string();
    while let Some(open) = opens.pop() {
        let close = if open == '{' { '}' } else { ']' };
        repaired.push(close);
    }
    serde_json::from_str::<Value>(&repaired).ok()
}

/// Try to parse a tool-call argument payload, returning either a parsed JSON value
/// or a string error message describing why parsing failed.
pub fn parse_tool_arguments(input: &str) -> Result<Value, String> {
    if let Some(v) = try_parse_lenient(input) {
        return Ok(v);
    }
    if let Some(v) = try_parse(input) {
        return Ok(v);
    }
    Err(format!(
        "Could not parse tool arguments from input of length {}",
        input.len()
    ))
}