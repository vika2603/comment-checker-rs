use std::collections::HashMap;
use std::ffi::c_void;
use std::path::{Path, PathBuf};
use std::process::Command;

use libloading::{Library, Symbol};

use crate::parser::languages::Language;

const DOWNLOAD_BASE_URL: &str =
    "https://github.com/vika2603/comment-checker-rs/releases/download";
const PARSERS_VERSION: &str = "parsers-v1";

pub struct GrammarCache {
    loaded: HashMap<String, LoadedGrammar>,
}

struct LoadedGrammar {
    _library: Library,
    language: tree_sitter::Language,
}

impl GrammarCache {
    pub fn new() -> Self {
        Self {
            loaded: HashMap::new(),
        }
    }

    pub fn get(
        &mut self,
        lang: Language,
        search_dirs: &[PathBuf],
    ) -> Result<tree_sitter::Language, String> {
        let key = lang.grammar_name();
        if let Some(entry) = self.loaded.get(key) {
            return Ok(entry.language.clone());
        }

        let so_name = lang.so_file_name();
        for dir in search_dirs {
            let path = dir.join(so_name);
            if path.exists() {
                match load_grammar_from_path(&path, lang) {
                    Ok(loaded) => {
                        let ts_lang = loaded.language.clone();
                        self.loaded.insert(key.to_string(), loaded);
                        return Ok(ts_lang);
                    }
                    Err(e) => {
                        eprintln!(
                            "warning: failed to load {} from {}: {}",
                            so_name,
                            path.display(),
                            e
                        );
                        continue;
                    }
                }
            }
        }

        Err(format!(
            "grammar '{}' not found in any search directory",
            lang.grammar_name()
        ))
    }

    pub fn is_loaded(&self, lang: Language) -> bool {
        self.loaded.contains_key(lang.grammar_name())
    }

    /// Return the already-cached Language without triggering a load.
    pub fn get_cached(&self, lang: Language) -> Option<&tree_sitter::Language> {
        self.loaded.get(lang.grammar_name()).map(|g| &g.language)
    }

    /// Build the ordered list of directories to search for grammar .so files.
    pub fn build_search_dirs(parser_config: &crate::config::ParserConfig) -> Vec<PathBuf> {
        let mut dirs = Vec::new();

        if let Some(ref path) = parser_config.path {
            dirs.push(path.clone());
        }

        if parser_config.use_nvim_parsers {
            if let Some(data_dir) = xdg_data_dir() {
                dirs.push(data_dir.join("nvim/site/parser"));
            }
        }

        if let Some(cache) = grammar_cache_dir() {
            dirs.push(cache);
        }

        dirs
    }

    /// Resolve a grammar: search dirs first, then optionally download.
    pub fn resolve(
        &mut self,
        lang: Language,
        parser_config: &crate::config::ParserConfig,
    ) -> Result<tree_sitter::Language, String> {
        let search_dirs = Self::build_search_dirs(parser_config);

        match self.get(lang, &search_dirs) {
            Ok(ts_lang) => return Ok(ts_lang),
            Err(_) if !parser_config.auto_download => {}
            Err(_) => {
                if let Some(cache_dir) = grammar_cache_dir() {
                    match download_grammar(lang.grammar_name(), &cache_dir) {
                        Ok(_) => {
                            return self.get(lang, &[cache_dir]);
                        }
                        Err(e) => {
                            eprintln!("warning: download failed for {}: {e}", lang.grammar_name());
                        }
                    }
                }
            }
        }

        Err(format!(
            "grammar '{}' not available (searched {} dirs, download {})",
            lang.grammar_name(),
            search_dirs.len(),
            if parser_config.auto_download { "attempted" } else { "disabled" }
        ))
    }
}

fn load_grammar_from_path(path: &Path, lang: Language) -> Result<LoadedGrammar, String> {
    let lib = unsafe { Library::new(path) }
        .map_err(|e| format!("dlopen {}: {}", path.display(), e))?;

    let ts_lang = unsafe {
        let func: Symbol<unsafe extern "C" fn() -> *const c_void> = lib
            .get(lang.symbol_name())
            .map_err(|e| format!("symbol lookup in {}: {}", path.display(), e))?;
        // Convert Symbol -> raw fn ptr -> LanguageFn -> Language
        // This pattern is verified working with libloading 0.8 + tree-sitter-language 0.1
        let lang_fn = tree_sitter_language::LanguageFn::from_raw(
            std::mem::transmute(func.into_raw().into_raw()),
        );
        let language: tree_sitter::Language = lang_fn.into();
        language
    };

    // Validate ABI by attempting to set the language on a parser
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&ts_lang)
        .map_err(|e| format!("ABI incompatible {}: {}", path.display(), e))?;

    Ok(LoadedGrammar {
        _library: lib,
        language: ts_lang,
    })
}

fn platform_suffix() -> Result<&'static str, String> {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("macos", "aarch64") => Ok("darwin-arm64"),
        ("macos", "x86_64") => Ok("darwin-x86_64"),
        ("linux", "x86_64") => Ok("linux-x86_64"),
        ("linux", "aarch64") => Ok("linux-aarch64"),
        (os, arch) => Err(format!("unsupported platform: {os}-{arch}")),
    }
}

fn download_url(grammar_name: &str) -> Result<String, String> {
    let suffix = platform_suffix()?;
    Ok(format!(
        "{DOWNLOAD_BASE_URL}/{PARSERS_VERSION}/tree-sitter-{grammar_name}-{suffix}.so"
    ))
}


pub fn download_grammar(
    grammar_name: &str,
    cache_dir: &Path,
) -> Result<PathBuf, String> {
    std::fs::create_dir_all(cache_dir)
        .map_err(|e| format!("cannot create cache dir {}: {e}", cache_dir.display()))?;

    let url = download_url(grammar_name)?;
    let tmp_path = cache_dir.join(format!("{grammar_name}.so.tmp"));
    let final_path = cache_dir.join(format!("{grammar_name}.so"));

    let output = Command::new("curl")
        .args([
            "--silent",
            "--fail",
            "--show-error",
            "--retry", "3",
            "-L",
            &url,
            "--output",
            &tmp_path.to_string_lossy(),
        ])
        .output()
        .map_err(|e| format!("failed to run curl: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let _ = std::fs::remove_file(&tmp_path);
        return Err(format!("download {url}: {stderr}"));
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o755);
        std::fs::set_permissions(&tmp_path, perms)
            .map_err(|e| format!("chmod {}: {e}", tmp_path.display()))?;
    }

    std::fs::rename(&tmp_path, &final_path)
        .map_err(|e| format!("rename {} -> {}: {e}", tmp_path.display(), final_path.display()))?;

    Ok(final_path)
}

fn xdg_data_dir() -> Option<PathBuf> {
    if let Ok(xdg) = std::env::var("XDG_DATA_HOME") {
        Some(PathBuf::from(xdg))
    } else {
        std::env::var("HOME").ok().map(|h| PathBuf::from(h).join(".local/share"))
    }
}

pub fn grammar_cache_dir() -> Option<PathBuf> {
    let base = if let Ok(xdg) = std::env::var("XDG_CACHE_HOME") {
        PathBuf::from(xdg)
    } else {
        let home = std::env::var("HOME").ok()?;
        PathBuf::from(home).join(".cache")
    };
    Some(base.join("comment-checker/parsers"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_from_nvim_if_available() {
        let nvim_parser_dir = std::env::var("HOME")
            .map(|h| PathBuf::from(h).join(".local/share/nvim/site/parser"))
            .unwrap_or_else(|_| PathBuf::from("/nonexistent"));
        if !nvim_parser_dir.join("rust.so").exists() {
            eprintln!("skipping test: nvim rust parser not found");
            return;
        }
        let mut cache = GrammarCache::new();
        let result = cache.get(Language::Rust, &[nvim_parser_dir]);
        assert!(result.is_ok(), "failed to load rust grammar: {:?}", result);

        let lang = result.unwrap();
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&lang).unwrap();
        let tree = parser.parse("fn main() {}", None).unwrap();
        assert_eq!(tree.root_node().kind(), "source_file");
    }

    #[test]
    fn test_cache_returns_same_language() {
        let nvim_parser_dir = std::env::var("HOME")
            .map(|h| PathBuf::from(h).join(".local/share/nvim/site/parser"))
            .unwrap_or_else(|_| PathBuf::from("/nonexistent"));
        if !nvim_parser_dir.join("rust.so").exists() {
            return;
        }
        let mut cache = GrammarCache::new();
        let dirs = vec![nvim_parser_dir];
        let lang1 = cache.get(Language::Rust, &dirs).unwrap();
        let lang2 = cache.get(Language::Rust, &dirs).unwrap();
        assert_eq!(lang1.node_kind_count(), lang2.node_kind_count());
    }

    #[test]
    fn test_missing_grammar_returns_error() {
        let mut cache = GrammarCache::new();
        let result = cache.get(Language::Rust, &[PathBuf::from("/nonexistent")]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[test]
    fn test_jsx_reuses_javascript() {
        assert_eq!(Language::Jsx.so_file_name(), Language::JavaScript.so_file_name());
        assert_eq!(Language::Jsx.symbol_name(), Language::JavaScript.symbol_name());
    }

    #[test]
    fn test_build_search_dirs_default() {
        let config = crate::config::ParserConfig::default();
        let dirs = GrammarCache::build_search_dirs(&config);
        assert!(!dirs.is_empty());
    }

    #[test]
    fn test_build_search_dirs_custom_path() {
        let config = crate::config::ParserConfig {
            path: Some(PathBuf::from("/custom/parsers")),
            use_nvim_parsers: false,
            auto_download: false,
        };
        let dirs = GrammarCache::build_search_dirs(&config);
        assert_eq!(dirs[0], PathBuf::from("/custom/parsers"));
    }

    #[test]
    fn test_download_url_format() {
        let url = download_url("rust").unwrap();
        assert!(url.contains("tree-sitter-rust-"));
        assert!(url.contains(".so"));
        assert!(url.starts_with("https://"));
    }

    #[test]
    fn test_platform_suffix() {
        let suffix = platform_suffix().unwrap();
        assert!(
            ["darwin-arm64", "darwin-x86_64", "linux-x86_64", "linux-aarch64"]
                .contains(&suffix)
        );
    }
}
