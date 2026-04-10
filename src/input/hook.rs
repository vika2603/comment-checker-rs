use std::ops::Range;
use std::path::PathBuf;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct HookPayload {
    pub tool_name: String,
    pub tool_input: ToolInput,
}

#[derive(Debug, Deserialize)]
pub struct ToolInput {
    pub file_path: PathBuf,
    pub content: Option<String>,
    pub old_string: Option<String>,
    pub new_string: Option<String>,
    pub edits: Option<Vec<EditEntry>>,
}

#[derive(Debug, Deserialize)]
pub struct EditEntry {
    pub old_string: Option<String>,
    pub new_string: Option<String>,
}

/// Represents a file to check, along with optional line ranges that were changed.
/// When `changed_ranges` is `None`, the entire file should be checked.
#[derive(Debug)]
pub struct HookTarget {
    pub file_path: PathBuf,
    pub changed_ranges: Option<Vec<Range<usize>>>,
}

/// Number of lines added as buffer around changed regions.
const LINE_BUFFER: usize = 3;

/// Parse a Claude Code hook JSON payload and produce a `HookTarget`.
/// Returns `Err` for invalid JSON; for unknown tool types checks the whole file.
pub fn parse_hook_input(json: &str) -> Result<HookTarget, String> {
    let payload: HookPayload =
        serde_json::from_str(json).map_err(|e| format!("invalid hook JSON: {e}"))?;

    let file_path = payload.tool_input.file_path.clone();
    let tool_name = payload.tool_name.as_str();

    let changed_ranges = match tool_name {
        "Write" | "Create" => None,

        "Edit" => {
            let new_string = payload.tool_input.new_string.as_deref().unwrap_or("");
            if new_string.is_empty() {
                None
            } else {
                let file_content = read_file(&file_path)?;
                find_changed_ranges(&file_content, &[new_string])
            }
        }

        "MultiEdit" => {
            let edits = payload.tool_input.edits.as_deref().unwrap_or(&[]);
            let new_strings: Vec<&str> = edits
                .iter()
                .filter_map(|e| e.new_string.as_deref())
                .filter(|s| !s.is_empty())
                .collect();

            if new_strings.is_empty() {
                None
            } else {
                let file_content = read_file(&file_path)?;
                find_changed_ranges(&file_content, &new_strings)
            }
        }

        _ => None,
    };

    Ok(HookTarget {
        file_path,
        changed_ranges,
    })
}

fn read_file(path: &PathBuf) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|e| format!("cannot read {}: {e}", path.display()))
}

/// Find the line ranges (1-based, with `LINE_BUFFER` padding) in `file_content`
/// where each string in `new_strings` appears.
/// Returns `None` if no matches are found (signals "check whole file").
pub fn find_changed_ranges(file_content: &str, new_strings: &[&str]) -> Option<Vec<Range<usize>>> {
    let lines: Vec<&str> = file_content.lines().collect();
    let total_lines = lines.len();
    let mut ranges: Vec<Range<usize>> = Vec::new();

    for needle in new_strings {
        // Count logical lines, not `\n`: "a\nb" without a trailing newline still spans 2 lines.
        let needle_line_count = needle.lines().count().max(1);
        let mut offset = 0;
        while let Some(rel_pos) = file_content[offset..].find(needle) {
            let byte_pos = offset + rel_pos;
            let before = &file_content[..byte_pos];
            let start_line = before.as_bytes().iter().filter(|&&b| b == b'\n').count() + 1;
            let end_line = start_line + needle_line_count - 1;

            let buffered_start = start_line.saturating_sub(LINE_BUFFER).max(1);
            let buffered_end = (end_line + LINE_BUFFER + 1).min(total_lines + 1);

            ranges.push(buffered_start..buffered_end);
            offset = byte_pos + needle.len().max(1);
        }
    }

    if ranges.is_empty() {
        return None;
    }

    Some(merge_ranges(ranges))
}

/// Merge overlapping or adjacent ranges into a minimal set of disjoint ranges.
pub fn merge_ranges(mut ranges: Vec<Range<usize>>) -> Vec<Range<usize>> {
    if ranges.is_empty() {
        return ranges;
    }

    ranges.sort_by_key(|r| r.start);

    let mut merged: Vec<Range<usize>> = Vec::new();
    let mut current = ranges[0].clone();

    for r in ranges.into_iter().skip(1) {
        if r.start <= current.end {
            current.end = current.end.max(r.end);
        } else {
            merged.push(current);
            current = r;
        }
    }
    merged.push(current);
    merged
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "line one\nline two\nline three\nline four\nline five\nline six\nline seven\nline eight\nline nine\nline ten\n";

    #[test]
    fn test_find_changed_ranges_single() {
        // "line four" is line 4; with buffer 3 => lines 1..8
        let ranges = find_changed_ranges(SAMPLE, &["line four"]).unwrap();
        assert_eq!(ranges.len(), 1);
        assert_eq!(ranges[0].start, 1); // max(4-3, 1) = 1
        assert_eq!(ranges[0].end, 8); // 4+3+1 = 8
    }

    #[test]
    fn test_find_changed_ranges_multiple_no_overlap() {
        // "line two" = line 2, "line nine" = line 9
        // line 2 -> buffered 1..6, line 9 -> buffered 6..11 => merged
        let ranges = find_changed_ranges(SAMPLE, &["line two", "line nine"]).unwrap();
        // Both ranges touch at 6, should merge into one
        assert!(ranges.len() <= 2);
    }

    #[test]
    fn test_find_changed_ranges_multiline_needle_no_trailing_newline() {
        // Guards against treating `\n` count as line count: this needle has
        // one `\n` but occupies two logical lines.
        let ranges = find_changed_ranges(SAMPLE, &["line three\nline four"]).unwrap();
        assert_eq!(ranges.len(), 1);
        assert_eq!(ranges[0].start, 1);
        assert_eq!(ranges[0].end, 8);
    }

    #[test]
    fn test_find_changed_ranges_not_found() {
        let result = find_changed_ranges(SAMPLE, &["does not exist"]);
        assert!(result.is_none());
    }

    #[test]
    fn test_merge_ranges_overlapping() {
        let ranges = vec![1..5, 3..8, 10..15];
        let merged = merge_ranges(ranges);
        assert_eq!(merged, vec![1..8, 10..15]);
    }

    #[test]
    fn test_merge_ranges_adjacent() {
        let ranges = vec![1..5, 5..10];
        let merged = merge_ranges(ranges);
        assert_eq!(merged, vec![1..10]);
    }

    #[test]
    fn test_merge_ranges_single() {
        let ranges = std::iter::once(2..7).collect();
        let merged = merge_ranges(ranges);
        assert_eq!(merged, vec![2..7]);
    }

    #[test]
    fn test_merge_ranges_empty() {
        let merged = merge_ranges(vec![]);
        assert!(merged.is_empty());
    }

    #[test]
    fn test_parse_hook_input_invalid_json() {
        let result = parse_hook_input("{not valid json}");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_hook_input_write_tool() {
        // Write tool => check whole file (None ranges)
        let json = r#"{
            "tool_name": "Write",
            "tool_input": {
                "file_path": "/tmp/nonexistent_test_file.rs",
                "content": "fn main() {}"
            }
        }"#;
        let result = parse_hook_input(json).unwrap();
        assert_eq!(
            result.file_path,
            PathBuf::from("/tmp/nonexistent_test_file.rs")
        );
        assert!(result.changed_ranges.is_none());
    }
}
