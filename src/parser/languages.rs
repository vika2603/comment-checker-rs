use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Language {
    Rust,
    Python,
    JavaScript,
    Jsx,
    TypeScript,
    Tsx,
    Go,
    Java,
    C,
    Cpp,
    Ruby,
    Shell,
}

impl Language {
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext {
            "rs" => Some(Language::Rust),
            "py" | "pyi" => Some(Language::Python),
            "js" | "mjs" | "cjs" => Some(Language::JavaScript),
            "jsx" => Some(Language::Jsx),
            "ts" | "mts" | "cts" => Some(Language::TypeScript),
            "tsx" => Some(Language::Tsx),
            "go" => Some(Language::Go),
            "java" => Some(Language::Java),
            "c" | "h" => Some(Language::C),
            "cpp" | "cc" | "cxx" | "hpp" | "hxx" => Some(Language::Cpp),
            "rb" => Some(Language::Ruby),
            "sh" | "bash" | "zsh" => Some(Language::Shell),
            _ => None,
        }
    }

    pub fn from_path(path: &Path) -> Option<Self> {
        path.extension()
            .and_then(|e| e.to_str())
            .and_then(Self::from_extension)
    }

    /// The .so file name to look for (e.g. "rust.so").
    /// JSX reuses javascript.so. All others match their grammar name.
    pub fn so_file_name(&self) -> &'static str {
        match self {
            Language::Rust => "rust.so",
            Language::Python => "python.so",
            Language::JavaScript | Language::Jsx => "javascript.so",
            Language::TypeScript => "typescript.so",
            Language::Tsx => "tsx.so",
            Language::Go => "go.so",
            Language::Java => "java.so",
            Language::C => "c.so",
            Language::Cpp => "cpp.so",
            Language::Ruby => "ruby.so",
            Language::Shell => "bash.so",
        }
    }

    /// The C symbol exported by the grammar .so (e.g. "tree_sitter_rust").
    pub fn symbol_name(&self) -> &'static [u8] {
        match self {
            Language::Rust => b"tree_sitter_rust",
            Language::Python => b"tree_sitter_python",
            Language::JavaScript | Language::Jsx => b"tree_sitter_javascript",
            Language::TypeScript => b"tree_sitter_typescript",
            Language::Tsx => b"tree_sitter_tsx",
            Language::Go => b"tree_sitter_go",
            Language::Java => b"tree_sitter_java",
            Language::C => b"tree_sitter_c",
            Language::Cpp => b"tree_sitter_cpp",
            Language::Ruby => b"tree_sitter_ruby",
            Language::Shell => b"tree_sitter_bash",
        }
    }

    /// The grammar name used in download URLs (e.g. "rust", "bash").
    pub fn grammar_name(&self) -> &'static str {
        match self {
            Language::Rust => "rust",
            Language::Python => "python",
            Language::JavaScript | Language::Jsx => "javascript",
            Language::TypeScript => "typescript",
            Language::Tsx => "tsx",
            Language::Go => "go",
            Language::Java => "java",
            Language::C => "c",
            Language::Cpp => "cpp",
            Language::Ruby => "ruby",
            Language::Shell => "bash",
        }
    }

    /// All distinct grammar names (for fetch-parsers --all).
    pub fn all_grammar_names() -> &'static [&'static str] {
        &[
            "rust", "python", "javascript", "typescript", "tsx",
            "go", "java", "c", "cpp", "ruby", "bash",
        ]
    }
}
