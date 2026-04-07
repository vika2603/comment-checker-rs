use std::path::PathBuf;

use ignore::WalkBuilder;

use crate::parser::languages::Language;

pub struct DiscoveredFile {
    pub path: PathBuf,
    pub language: Language,
}

/// Walk `paths` (files and/or directories), respecting `.gitignore`.
///
/// Each file is included only when:
/// - Its extension maps to a known `Language`
/// - Its size is <= `max_file_size`
/// - It contains no NUL bytes in the first 8192 bytes (binary check)
/// - Its language name is in `language_filter` (or `language_filter` is empty)
pub fn discover_files(
    paths: &[PathBuf],
    max_file_size: u64,
    language_filter: &[String],
) -> Vec<DiscoveredFile> {
    let mut results = Vec::new();

    for root in paths {
        if root.is_file() {
            if let Some(file) = check_file(root, max_file_size, language_filter) {
                results.push(file);
            }
            continue;
        }

        let walker = WalkBuilder::new(root)
            .hidden(false)
            .git_ignore(true)
            .git_global(true)
            .git_exclude(true)
            .build();

        for entry in walker.flatten() {
            let path = entry.path().to_path_buf();
            if path.is_file() {
                if let Some(file) = check_file(&path, max_file_size, language_filter) {
                    results.push(file);
                }
            }
        }
    }

    results
}

fn check_file(
    path: &PathBuf,
    max_file_size: u64,
    language_filter: &[String],
) -> Option<DiscoveredFile> {
    let language = Language::from_path(path)?;

    // Apply language filter if specified
    if !language_filter.is_empty() && !language_filter.iter().any(|f| f == language.name()) {
        return None;
    }

    // Check file size
    let metadata = std::fs::metadata(path).ok()?;
    if metadata.len() > max_file_size {
        return None;
    }

    // Binary check: look for NUL bytes in first 8192 bytes
    if is_binary(path) {
        return None;
    }

    Some(DiscoveredFile {
        path: path.clone(),
        language,
    })
}

fn is_binary(path: &PathBuf) -> bool {
    use std::io::Read;
    let Ok(mut file) = std::fs::File::open(path) else {
        return false;
    };
    let mut buf = [0u8; 8192];
    let n = file.read(&mut buf).unwrap_or(0);
    buf[..n].contains(&0)
}
