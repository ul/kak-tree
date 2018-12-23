extern crate cc;

use std::path::PathBuf;

fn main() {
    let tree_sitter_rust: PathBuf = ["vendor", "tree-sitter-rust", "src"].iter().collect();

    cc::Build::new()
        .include(&tree_sitter_rust)
        .file(tree_sitter_rust.join("parser.c"))
        .file(tree_sitter_rust.join("scanner.c"))
        .compile("tree_sitter_rust");
}
