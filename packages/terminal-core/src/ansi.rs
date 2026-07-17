// Terminal Core module implements ANSI escape handling.
// 翻译自 packages/terminal-core/src/ansi.ts

use once_cell::sync::Lazy;
use regex::Regex;
use unicode_segmentation::UnicodeSegmentation;

// Full CSI: ESC [ <params> <final byte> covers cursor movement, erase, and SGR.
const ESC_ANSI_CSI_PATTERN: &str = "\\x1b\\[[\\x20-\\x3f]*[\\x40-\\x7e]";
const C1_ANSI_CSI_PATTERN: &str = "\\x9b[\\x20-\\x3f]*[\\x40-\\x7e]";
const ANSI_CSI_PATTERN: &str = "(?:__ESC__|__C1__)";
// OSC: ESC ] or C1 OSC, then <payload> ST.
const ANSI_OSC_INTRODUCER_PATTERN: &str = "(?:\\x1b\\]|\\x9d)";
const ANSI_STRING_TERMINATOR_PATTERN: &str = "(?:\\x1b\\\\|\\x07|\\x9c)";

fn ansi_osc_pattern() -> String {
    format!(
        "{}[^\\x07\\x1b\\x9c]{}",
        ANSI_OSC_INTRODUCER_PATTERN, ANSI_STRING_TERMINATOR_PATTERN
    )
}

fn ansi_sequence_pattern() -> String {
    let csi = ANSI_CSI_PATTERN
        .replace("__ESC__", ESC_ANSI_CSI_PATTERN)
        .replace("__C1__", C1_ANSI_CSI_PATTERN);
    format!("(?:{})|(?:{})", ansi_osc_pattern(), csi)
}

fn ansi_compat_osc_sequence_pattern() -> String {
    format!(
        "{}[\\s\\S]*?{}",
        ANSI_OSC_INTRODUCER_PATTERN, ANSI_STRING_TERMINATOR_PATTERN
    )
}

const ANSI_CONTROL_SEQUENCE_PATTERN: &str =
    "[\\u001B\\u009B][[\\]()#;?]*(?:\\d{1,4}(?:[;:]\\d{0,4})*)?[\\dA-PR-TZcf-nq-uy=><~]";

fn ansi_compat_sequence_pattern() -> String {
    format!(
        "(?:{})|(?:{})",
        ansi_compat_osc_sequence_pattern(),
        ANSI_CONTROL_SEQUENCE_PATTERN
    )
}

struct CompiledRegexes {
    osc_at_index: Regex,
    sequence_regex: Regex,
    compat_seq_at_index: Regex,
}

static REGEXES: Lazy<CompiledRegexes> = Lazy::new(|| {
    let osc = Regex::new(&ansi_osc_pattern()).expect("ANSI OSC pattern");
    let seq = Regex::new(&ansi_sequence_pattern()).expect("ANSI sequence pattern");
    let compat = Regex::new(&ansi_compat_sequence_pattern()).expect("ANSI compat sequence pattern");
    CompiledRegexes {
        osc_at_index: osc,
        sequence_regex: seq,
        compat_seq_at_index: compat,
    }
});

fn csi_introducer_length(input: &str, index: usize) -> usize {
    let bytes = input.as_bytes();
    let code = bytes[index];
    if code == 0x9b {
        return 1;
    }
    if code == 0x1b && index + 1 < bytes.len() && bytes[index + 1] == 0x5b {
        return 2;
    }
    0
}

fn has_ansi_introducer(input: &str) -> bool {
    input.contains('\u{001B}') || input.contains('\u{009B}') || input.contains('\u{009D}')
}

fn find_osc_at(input: &str, index: usize) -> Option<(usize, usize)> {
    let re = &REGEXES.osc_at_index;
    let mat = re.find_at(input, index)?;
    Some((mat.start(), mat.end()))
}

fn find_compat_at(input: &str, index: usize) -> Option<(usize, usize)> {
    let re = &REGEXES.compat_seq_at_index;
    let mat = re.find_at(input, index)?;
    Some((mat.start(), mat.end()))
}

struct StripOptions {
    compatibility_grammar: bool,
    preserve_incomplete_csi: bool,
}

/**
 * Strip ANSI against original input positions so one removal cannot synthesize
 * a second sequence. C0 controls execute without ending CSI, CAN/SUB cancel it,
 * and ESC restarts escape parsing.
 */
fn strip_ansi_internal(input: &str, options: StripOptions) -> String {
    let mut output = String::new();
    let mut copy_start: usize = 0;
    let mut index: usize = 0;
    let bytes = input.as_bytes();
    let len = input.len();

    while index < len {
        let introducer_code = bytes[index];
        if introducer_code != 0x1b && introducer_code != 0x9b && introducer_code != 0x9d {
            index += 1;
            continue;
        }

        // OSC?
        if let Some((start, end)) = find_osc_at(input, index) {
            if start == index {
                output.push_str(&input[copy_start..index]);
                index = end;
                copy_start = index;
                continue;
            }
        }

        let introducer_length = csi_introducer_length(input, index);
        if introducer_length == 0 {
            if options.compatibility_grammar {
                if let Some((start, end)) = find_compat_at(input, index) {
                    if start == index {
                        output.push_str(&input[copy_start..index]);
                        index = end;
                        copy_start = index;
                        continue;
                    }
                }
            }
            index += 1;
            continue;
        }

        let compatibility_match = if options.compatibility_grammar {
            find_compat_at(input, index)
        } else {
            None
        };

        let mut cursor = index + introducer_length;
        let mut controls: Vec<u8> = Vec::new();
        let mut ended = false;
        while cursor < len {
            let code = bytes[cursor];
            if code == 0x18 || code == 0x1a {
                cursor += 1;
                ended = true;
                break;
            }
            if code == 0x1b || code == 0x9b {
                ended = true;
                break;
            }
            if code <= 0x1f || code == 0x7f {
                controls.push(code);
                cursor += 1;
                continue;
            }
            if code >= 0x20 && code <= 0x3f {
                cursor += 1;
                continue;
            }
            if code >= 0x40 && code <= 0x7e {
                cursor += 1;
            }
            ended = true;
            break;
        }

        if !ended && options.preserve_incomplete_csi {
            break;
        }

        let canonical_length = cursor - index;
        if controls.is_empty() {
            if let Some((_, end)) = compatibility_match {
                if end - index > canonical_length {
                    cursor = end;
                }
            }
        }

        output.push_str(&input[copy_start..index]);
        for c in &controls {
            output.push(*c as char);
        }
        index = cursor;
        copy_start = cursor;
    }

    output.push_str(&input[copy_start..]);
    output
}

pub fn strip_ansi(input: &str) -> String {
    if !has_ansi_introducer(input) {
        return input.to_string();
    }
    strip_ansi_internal(
        input,
        StripOptions {
            compatibility_grammar: false,
            preserve_incomplete_csi: false,
        },
    )
}

pub fn strip_ansi_sequences(input: &str) -> String {
    if !has_ansi_introducer(input) {
        return input.to_string();
    }
    strip_ansi_internal(
        input,
        StripOptions {
            compatibility_grammar: true,
            preserve_incomplete_csi: false,
        },
    )
}

/// Preserve pending CSI visibly because an output chunk boundary is not true EOF.
pub fn strip_ansi_for_stream_chunk(input: &str, compatibility_grammar: Option<bool>) -> String {
    if !has_ansi_introducer(input) {
        return input.to_string();
    }
    strip_ansi_internal(
        input,
        StripOptions {
            compatibility_grammar: compatibility_grammar.unwrap_or(false),
            preserve_incomplete_csi: true,
        },
    )
}

pub fn split_graphemes(input: &str) -> Vec<String> {
    if input.is_empty() {
        return vec![];
    }
    input.graphemes(true).map(|s| s.to_string()).collect()
}

pub fn sanitize_for_log(v: &str) -> String {
    let stripped = strip_ansi(v);
    let mut out = String::with_capacity(stripped.len());
    for ch in stripped.chars() {
        let code = ch as u32;
        let is_control = (code <= 0x1f) || code == 0x7f || (code >= 0x80 && code <= 0x9f);
        if !is_control {
            out.push(ch);
        }
    }
    out
}

fn is_zero_width_code_point(code_point: u32) -> bool {
    (code_point >= 0x0300 && code_point <= 0x036f)
        || (code_point >= 0x1ab0 && code_point <= 0x1aff)
        || (code_point >= 0x1dc0 && code_point <= 0x1dff)
        || (code_point >= 0x20d0 && code_point <= 0x20ff)
        || (code_point >= 0xfe20 && code_point <= 0xfe2f)
        || (code_point >= 0xfe00 && code_point <= 0xfe0f)
        || code_point == 0x200d
}

fn is_full_width_code_point(code_point: u32) -> bool {
    if code_point < 0x1100 {
        return false;
    }
    code_point <= 0x115f
        || code_point == 0x2329
        || code_point == 0x232a
        || (code_point >= 0x2e80 && code_point <= 0x3247 && code_point != 0x303f)
        || (code_point >= 0x3250 && code_point <= 0x4dbf)
        || (code_point >= 0x4e00 && code_point <= 0xa4c6)
        || (code_point >= 0xa960 && code_point <= 0xa97c)
        || (code_point >= 0xac00 && code_point <= 0xd7a3)
        || (code_point >= 0xf900 && code_point <= 0xfaff)
        || (code_point >= 0xfe10 && code_point <= 0xfe19)
        || (code_point >= 0xfe30 && code_point <= 0xfe6b)
        || (code_point >= 0xff01 && code_point <= 0xff60)
        || (code_point >= 0xffe0 && code_point <= 0xffe6)
        || (code_point >= 0x1aff0 && code_point <= 0x1aff3)
        || (code_point >= 0x1aff5 && code_point <= 0x1affb)
        || (code_point >= 0x1affd && code_point <= 0x1affe)
        || (code_point >= 0x1b000 && code_point <= 0x1b2ff)
        || (code_point >= 0x1f200 && code_point <= 0x1f251)
        || (code_point >= 0x20000 && code_point <= 0x3fffd)
}

static EXTENDED_PICTOGRAPHIC_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\p{Extended_Pictographic}").unwrap());

fn is_wide_emoji_grapheme(grapheme: &str) -> bool {
    if grapheme.is_empty() {
        return false;
    }
    let chars: Vec<char> = grapheme.chars().collect();
    let regional_count = chars
        .iter()
        .filter(|c| {
            let cp = **c as u32;
            cp >= 0x1F1E6 && cp <= 0x1F1FF
        })
        .count();
    if regional_count > 0 {
        return regional_count >= 2;
    }
    let is_pictographic =
        EXTENDED_PICTOGRAPHIC_RE.is_match(grapheme) || grapheme.contains('\u{20E3}');
    if is_pictographic {
        return true;
    }
    if is_unqualified_keycap(grapheme) {
        return true;
    }
    if grapheme.contains('\u{200D}') {
        let pictographic_count = chars
            .iter()
            .filter(|c| EXTENDED_PICTOGRAPHIC_RE.is_match(&c.to_string()))
            .count();
        return pictographic_count >= 2;
    }
    false
}

fn is_unqualified_keycap(grapheme: &str) -> bool {
    let chars: Vec<char> = grapheme.chars().collect();
    if chars.len() != 2 {
        return false;
    }
    let base = chars[0] as u32;
    let keycap = chars[1] as u32;
    (base == 0x23 || base == 0x2A || (0x30..=0x39).contains(&base)) && keycap == 0x20E3
}

fn grapheme_width(grapheme: &str) -> usize {
    if grapheme.is_empty() {
        return 0;
    }
    if is_wide_emoji_grapheme(grapheme) {
        return 2;
    }

    let mut saw_printable = false;
    for ch in grapheme.chars() {
        let code_point = ch as u32;
        if is_zero_width_code_point(code_point) {
            continue;
        }
        if is_full_width_code_point(code_point) {
            return 2;
        }
        saw_printable = true;
    }
    if saw_printable {
        1
    } else {
        0
    }
}

pub fn visible_width(input: &str) -> usize {
    split_graphemes(&strip_ansi(input))
        .iter()
        .map(|g| grapheme_width(g))
        .sum()
}

/// Truncate to at most `max_width` visible columns, dropping whole grapheme
/// clusters that would overflow while preserving ANSI sequences verbatim.
pub fn truncate_to_visible_width(input: &str, max_width: usize) -> String {
    if max_width == 0 {
        return String::new();
    }
    if visible_width(input) <= max_width {
        return input.to_string();
    }
    let re = &REGEXES.sequence_regex;
    let mut out = String::new();
    let mut used: usize = 0;
    let mut pos: usize = 0;
    let mut budget_spent = false;

    for mat in re.find_iter(input) {
        append_visible(
            &input[pos..mat.start()],
            &mut out,
            &mut used,
            &mut budget_spent,
            max_width,
        );
        out.push_str(mat.as_str());
        pos = mat.end();
    }
    append_visible(
        &input[pos..],
        &mut out,
        &mut used,
        &mut budget_spent,
        max_width,
    );
    out
}

fn append_visible(
    segment: &str,
    out: &mut String,
    used: &mut usize,
    budget_spent: &mut bool,
    max_width: usize,
) {
    if *budget_spent {
        return;
    }
    for grapheme in split_graphemes(segment) {
        let width = grapheme_width(&grapheme);
        if *used + width > max_width {
            *budget_spent = true;
            return;
        }
        out.push_str(&grapheme);
        *used += width;
    }
}
