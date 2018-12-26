#[macro_use]
extern crate slog;
#[macro_use]
extern crate slog_scope;

use clap::{crate_version, App, Arg};
use itertools::Itertools;
use serde::Deserialize;
use sloggers::terminal::{Destination, TerminalLoggerBuilder};
use sloggers::types::Severity;
use sloggers::Build;
use std::io::Read;
use toml;
use tree_sitter::{Language, Node, Parser, Point, Range, Tree};

extern "C" {
    fn tree_sitter_bash() -> Language;
    fn tree_sitter_c() -> Language;
    fn tree_sitter_c_sharp() -> Language;
    fn tree_sitter_cpp() -> Language;
    fn tree_sitter_css() -> Language;
    fn tree_sitter_go() -> Language;
    fn tree_sitter_haskell() -> Language;
    fn tree_sitter_html() -> Language;
    fn tree_sitter_java() -> Language;
    fn tree_sitter_javascript() -> Language;
    fn tree_sitter_json() -> Language;
    fn tree_sitter_julia() -> Language;
    fn tree_sitter_ocaml() -> Language;
    fn tree_sitter_php() -> Language;
    fn tree_sitter_python() -> Language;
    fn tree_sitter_ruby() -> Language;
    fn tree_sitter_rust() -> Language;
    fn tree_sitter_scala() -> Language;
    // fn tree_sitter_swift() -> Language;
    fn tree_sitter_typescript() -> Language;
}

#[derive(Deserialize)]
enum Op {
    SelectNode,
}

#[derive(Deserialize)]
struct Request {
    op: Op,
    filetype: String,
    selections_desc: String,
    content: String,
}

fn main() {
    let matches = App::new("kak-tree")
        .version(crate_version!())
        .author("Ruslan Prokopchuk <fer.obbee@gmail.com>")
        .about("Structural selections for Kakoune")
        .arg(
            Arg::with_name("v")
                .short("v")
                .multiple(true)
                .help("Sets the level of verbosity"),
        )
        .get_matches();

    let verbosity = matches.occurrences_of("v") as u8;

    let level = match verbosity {
        0 => Severity::Error,
        1 => Severity::Warning,
        2 => Severity::Info,
        3 => Severity::Debug,
        _ => Severity::Trace,
    };

    let mut builder = TerminalLoggerBuilder::new();
    builder.level(level);
    builder.destination(Destination::Stderr);
    let logger = builder.build().unwrap();
    let _guard = slog_scope::set_global_logger(logger);

    let mut request = String::new();
    std::io::stdin().read_to_string(&mut request).unwrap();
    let request: Request = toml::from_str(&request).unwrap();
    let response = handle_request(&request);
    println!("{}", response);
}

fn handle_request(request: &Request) -> String {
    match &request.op {
        Op::SelectNode => {
            let mut parser = Parser::new();
            let language = filetype_to_language(&request.filetype);
            parser.set_language(language).unwrap();
            let tree = parser.parse_str(&request.content, None).unwrap();
            let buffer = request
                .content
                .split('\n')
                .map(|s| format!("{}\n", s))
                .collect::<Vec<_>>();
            let ranges = selections_desc_to_ranges(&buffer, &request.selections_desc);
            let mut new_ranges = Vec::new();
            for range in &ranges {
                let node = find_deepest_node_containing_range(&tree, range);
                new_ranges.push(node.range());
            }
            let selections_desc = ranges_to_selections_desc(&buffer, &new_ranges);
            format!("select {}", selections_desc)
        }
    }
}

fn find_deepest_node_containing_range<'a>(tree: &'a Tree, range: &Range) -> Node<'a> {
    let root = tree.root_node();
    let mut node = root;
    'outer: while node.range().start_byte <= range.start_byte
        && range.end_byte <= node.range().end_byte
    {
        let parent = node;
        for child in parent.children() {
            if child.is_named()
                && child.range().start_byte <= range.start_byte
                && range.end_byte < child.range().end_byte
                && !(child.range().start_byte == range.start_byte
                    && range.end_byte == child.range().end_byte - 1)
            {
                node = child;
                continue 'outer;
            }
        }
        return node;
    }
    root
}

fn ranges_to_selections_desc(buffer: &[String], ranges: &[Range]) -> String {
    ranges
        .iter()
        .map(|range| {
            let mut end_row = range.end_point.row;
            let mut end_column = range.end_point.column;
            if end_column > 0 {
                end_column -= 1;
            } else {
                end_row -= 1;
                end_column = 1_000_000;
            }
            format!(
                "{},{}",
                point_to_kak_coords(buffer, range.start_point),
                point_to_kak_coords(buffer, Point::new(end_row, end_column))
            )
        })
        .join(" ")
}

fn selections_desc_to_ranges(buffer: &[String], selections_desc: &str) -> Vec<Range> {
    selections_desc
        .split_whitespace()
        .map(|selection_desc| selection_desc_to_range(buffer, selection_desc))
        .collect()
}

fn selection_desc_to_range(buffer: &[String], selection_desc: &str) -> Range {
    let mut range = selection_desc.split(',');
    let start = range.next().unwrap();
    let end = range.next().unwrap();
    let (start_byte, start_point) = kak_coords_to_byte_and_point(buffer, start);
    let (end_byte, end_point) = kak_coords_to_byte_and_point(buffer, end);
    let reverse = start_byte > end_byte;
    if reverse {
        Range {
            start_byte: end_byte,
            end_byte: start_byte,
            start_point: end_point,
            end_point: start_point,
        }
    } else {
        Range {
            start_byte,
            end_byte,
            start_point,
            end_point,
        }
    }
}

fn point_to_kak_coords(buffer: &[String], p: Point) -> String {
    let offset = buffer[p.row]
        .char_indices()
        .enumerate()
        .find_map(|(column, (offset, _))| {
            if column == p.column {
                Some(offset)
            } else {
                None
            }
        })
        .unwrap();
    format!("{}.{}", p.row + 1, offset + 1)
}

fn kak_coords_to_byte_and_point(buffer: &[String], coords: &str) -> (usize, Point) {
    let mut coords = coords.split('.');
    let row = coords.next().unwrap().parse::<usize>().unwrap() - 1;
    let offset = coords.next().unwrap().parse::<usize>().unwrap() - 1;
    let byte = buffer[..row].iter().fold(0, |offset, c| offset + c.len()) + offset;
    let column = buffer[row]
        .char_indices()
        .position(|(i, _)| i == offset)
        .unwrap();
    (byte, Point::new(row, column))
}

fn filetype_to_language(filetype: &str) -> Language {
    let sitter = match filetype {
        "bash" => tree_sitter_bash,
        "c" => tree_sitter_c,
        "c_sharp" => tree_sitter_c_sharp,
        "cpp" => tree_sitter_cpp,
        "css" => tree_sitter_css,
        "go" => tree_sitter_go,
        "haskell" => tree_sitter_haskell,
        "html" => tree_sitter_html,
        "java" => tree_sitter_java,
        "javascript" => tree_sitter_javascript,
        "json" => tree_sitter_json,
        "julia" => tree_sitter_julia,
        "ocaml" => tree_sitter_ocaml,
        "php" => tree_sitter_php,
        "python" => tree_sitter_python,
        "ruby" => tree_sitter_ruby,
        "rust" => tree_sitter_rust,
        "scala" => tree_sitter_scala,
        // "swift" => tree_sitter_swift,
        "typescript" => tree_sitter_typescript,
        _ => unreachable!(),
    };
    unsafe { sitter() }
}
