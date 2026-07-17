// Media Understanding Common helper module supports format behavior.
// 翻译自 packages/media-understanding-common/src/format.ts

use crate::types::{MediaUnderstandingKind, MediaUnderstandingOutput};

const MEDIA_PLACEHOLDER_TOKEN: &str = r"<media:[^>]+>(?:\s*\([^)]*\))?";

fn media_placeholder_re() -> regex::Regex {
    regex::Regex::new(&format!(r"^(?:{}\s*)+$", MEDIA_PLACEHOLDER_TOKEN)).unwrap()
}

fn media_placeholder_token_re() -> regex::Regex {
    regex::Regex::new(&format!(r"^(?:{}\s*)+", MEDIA_PLACEHOLDER_TOKEN)).unwrap()
}

/** Extracts user-authored text while ignoring synthetic media placeholder tokens. */
pub fn extract_media_user_text(body: Option<&str>) -> Option<String> {
    let trimmed = body.map(|s| s.trim()).unwrap_or("");
    if trimmed.is_empty() {
        return None;
    }
    if media_placeholder_re().is_match(trimmed) {
        return None;
    }
    let cleaned = media_placeholder_token_re().replace_all(trimmed, "").trim().to_string();
    if cleaned.is_empty() {
        None
    } else {
        Some(cleaned)
    }
}

fn format_section(title: &str, kind: &str, text: &str, user_text: Option<&str>) -> String {
    let mut lines: Vec<String> = vec![format!("[{}]", title)];
    if let Some(ut) = user_text {
        lines.push(format!("User text:\n{}", ut));
    }
    lines.push(format!("{}:\n{}", kind, text));
    lines.join("\n")
}

/** Formats media-understanding outputs into the chat body sent back to the model. */
pub fn format_media_understanding_body(params: FormatMediaUnderstandingBodyParams) -> String {
    let outputs: Vec<&MediaUnderstandingOutput> = params
        .outputs
        .iter()
        .filter(|o| !o.text.trim().is_empty())
        .collect();
    if outputs.is_empty() {
        return params.body.clone().unwrap_or_default();
    }

    let user_text = extract_media_user_text(params.body.as_deref());
    let mut sections: Vec<String> = Vec::new();
    if user_text.is_some() && outputs.len() > 1 {
        sections.push(format!("User text:\n{}", user_text.as_deref().unwrap_or("")));
    }

    let mut counts: std::collections::HashMap<MediaUnderstandingKind, usize> =
        std::collections::HashMap::new();
    for output in &outputs {
        *counts.entry(output.kind).or_insert(0) += 1;
    }
    let mut seen: std::collections::HashMap<MediaUnderstandingKind, usize> =
        std::collections::HashMap::new();

    for output in &outputs {
        let count = counts.get(&output.kind).copied().unwrap_or(1);
        let next = seen.get(&output.kind).copied().unwrap_or(0) + 1;
        seen.insert(output.kind, next);
        let suffix = if count > 1 { format!(" {}/{}", next, count) } else { String::new() };
        let only_user_text = if outputs.len() == 1 { user_text.as_deref() } else { None };
        match output.kind {
            MediaUnderstandingKind::AudioTranscription => {
                sections.push(format_section(
                    &format!("Audio{}", suffix),
                    "Transcript",
                    &output.text,
                    only_user_text,
                ));
            }
            MediaUnderstandingKind::ImageDescription => {
                sections.push(format_section(
                    &format!("Image{}", suffix),
                    "Description",
                    &output.text,
                    only_user_text,
                ));
            }
            MediaUnderstandingKind::VideoDescription => {
                sections.push(format_section(
                    &format!("Video{}", suffix),
                    "Description",
                    &output.text,
                    only_user_text,
                ));
            }
        }
    }

    sections.join("\n\n").trim().to_string()
}

#[derive(Debug, Clone, Default)]
pub struct FormatMediaUnderstandingBodyParams {
    pub body: Option<String>,
    pub outputs: Vec<MediaUnderstandingOutput>,
}

/** Formats one or more audio transcript outputs for legacy transcript-only callers. */
pub fn format_audio_transcripts(outputs: &[MediaUnderstandingOutput]) -> String {
    if outputs.len() == 1 {
        let output = &outputs[0];
        return output.text.clone();
    }
    outputs
        .iter()
        .enumerate()
        .map(|(index, output)| format!("Audio {}:\n{}", index + 1, output.text))
        .collect::<Vec<_>>()
        .join("\n\n")
}