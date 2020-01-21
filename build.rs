extern crate cc;

use std::path::PathBuf;

fn main() {
    for lang in &[
        #[cfg(feature = "bash")]
        "bash",
        #[cfg(feature = "c_sharp")]
        "c-sharp",
        #[cfg(feature = "c")]
        "c",
        #[cfg(feature = "cpp")]
        "cpp",
        #[cfg(feature = "css")]
        "css",
        #[cfg(feature = "elm")]
        "elm",
        #[cfg(feature = "go")]
        "go",
        #[cfg(feature = "haskell")]
        "haskell",
        #[cfg(feature = "html")]
        "html",
        #[cfg(feature = "java")]
        "java",
        #[cfg(feature = "javascript")]
        "javascript",
        #[cfg(feature = "json")]
        "json",
        #[cfg(feature = "julia")]
        "julia",
        #[cfg(feature = "ocaml")]
        "ocaml",
        #[cfg(feature = "php")]
        "php",
        #[cfg(feature = "python")]
        "python",
        #[cfg(feature = "ruby")]
        "ruby",
        #[cfg(feature = "rust")]
        "rust",
        #[cfg(feature = "scala")]
        "scala",
        #[cfg(feature = "typescript")]
        "typescript",
    ] {
        let mut build = cc::Build::new();

        let tree_sitter: PathBuf = match *lang {
            "typescript" => [
                "vendor",
                &format!("tree-sitter-{}", lang),
                "typescript",
                "src",
            ]
            .iter()
            .collect(),
            _ => ["vendor", &format!("tree-sitter-{}", lang), "src"]
                .iter()
                .collect(),
        };

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
