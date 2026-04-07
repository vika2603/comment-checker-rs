use std::collections::HashMap;
use std::ffi::c_void;
use std::path::{Path, PathBuf};

use libloading::{Library, Symbol};

use crate::parser::languages::Language;

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
}
