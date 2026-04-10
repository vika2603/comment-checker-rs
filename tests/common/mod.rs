use comment_checker::grammar::{GrammarCache, grammar_cache_dir};
use comment_checker::parser::languages::Language;

pub fn load_ts_language(lang: Language) -> Option<tree_sitter::Language> {
    let cache_dir = grammar_cache_dir()?;
    let cache = Box::leak(Box::new(GrammarCache::new()));
    cache.get(lang, &[cache_dir]).ok()
}
