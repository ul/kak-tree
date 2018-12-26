extern crate cc;

use std::path::PathBuf;

fn main() {
    for lang in &[
        "bash",
        "c-sharp",
        "c",
        "cpp",
        "css",
        "go",
        "haskell",
        "html",
        "java",
        "javascript",
        "json",
        "julia",
        "ocaml",
        "php",
        "python",
        "ruby",
        "rust",
        "scala",
        // "swift",
        "typescript",
    ] {
        let mut build = cc::Build::new();

        let tree_sitter: PathBuf = ["vendor", &format!("tree-sitter-{}", lang), "src"]
            .iter()
            .collect();

        build
            .include(&tree_sitter)
            .file(tree_sitter.join("parser.c"));

        if tree_sitter.join("scanner.c").exists() {
            build.file(tree_sitter.join("scanner.c"));
        } else if tree_sitter.join("scanner.cc").exists() {
            let mut build = cc::Build::new();
            build.include(&tree_sitter);
            build.cpp(true);
            build.file(tree_sitter.join("scanner.cc"));
            build.compile(&format!("tree_sitter_{}_scanner", lang));
        };

        build.compile(&format!("tree_sitter_{}", lang));
    }
}
