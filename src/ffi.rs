use tree_sitter::Language;

extern "C" {
    #[cfg(feature = "bash")]
    fn tree_sitter_bash() -> Language;
    #[cfg(feature = "c")]
    fn tree_sitter_c() -> Language;
    #[cfg(feature = "c_sharp")]
    fn tree_sitter_c_sharp() -> Language;
    #[cfg(feature = "cpp")]
    fn tree_sitter_cpp() -> Language;
    #[cfg(feature = "css")]
    fn tree_sitter_css() -> Language;
    #[cfg(feature = "go")]
    fn tree_sitter_go() -> Language;
    #[cfg(feature = "haskell")]
    fn tree_sitter_haskell() -> Language;
    #[cfg(feature = "html")]
    fn tree_sitter_html() -> Language;
    #[cfg(feature = "java")]
    fn tree_sitter_java() -> Language;
    #[cfg(feature = "javascript")]
    fn tree_sitter_javascript() -> Language;
    #[cfg(feature = "json")]
    fn tree_sitter_json() -> Language;
    #[cfg(feature = "julia")]
    fn tree_sitter_julia() -> Language;
    #[cfg(feature = "ocaml")]
    fn tree_sitter_ocaml() -> Language;
    #[cfg(feature = "php")]
    fn tree_sitter_php() -> Language;
    #[cfg(feature = "python")]
    fn tree_sitter_python() -> Language;
    #[cfg(feature = "ruby")]
    fn tree_sitter_ruby() -> Language;
    #[cfg(feature = "rust")]
    fn tree_sitter_rust() -> Language;
    #[cfg(feature = "scala")]
    fn tree_sitter_scala() -> Language;
    #[cfg(feature = "typescript")]
    fn tree_sitter_typescript() -> Language;
}

pub fn filetype_to_language(filetype: &str) -> Option<Language> {
    let sitter = match filetype {
        #[cfg(feature = "bash")]
        "sh" => tree_sitter_bash,
        #[cfg(feature = "c")]
        "c" => tree_sitter_c,
        #[cfg(feature = "c_sharp")]
        "c_sharp" => tree_sitter_c_sharp,
        #[cfg(feature = "cpp")]
        "cpp" => tree_sitter_cpp,
        #[cfg(feature = "css")]
        "css" => tree_sitter_css,
        #[cfg(feature = "go")]
        "go" => tree_sitter_go,
        #[cfg(feature = "haskell")]
        "haskell" => tree_sitter_haskell,
        #[cfg(feature = "html")]
        "html" => tree_sitter_html,
        #[cfg(feature = "java")]
        "java" => tree_sitter_java,
        #[cfg(feature = "javascript")]
        "javascript" => tree_sitter_javascript,
        #[cfg(feature = "json")]
        "json" => tree_sitter_json,
        #[cfg(feature = "julia")]
        "julia" => tree_sitter_julia,
        #[cfg(feature = "ocaml")]
        "ocaml" => tree_sitter_ocaml,
        #[cfg(feature = "php")]
        "php" => tree_sitter_php,
        #[cfg(feature = "python")]
        "python" => tree_sitter_python,
        #[cfg(feature = "ruby")]
        "ruby" => tree_sitter_ruby,
        #[cfg(feature = "rust")]
        "rust" => tree_sitter_rust,
        #[cfg(feature = "scala")]
        "scala" => tree_sitter_scala,
        #[cfg(feature = "typescript")]
        "typescript" => tree_sitter_typescript,
        _ => return None,
    };
    Some(unsafe { sitter() })
}
