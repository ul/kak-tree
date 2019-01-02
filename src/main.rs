use crate::config::{Config, FiletypeConfig};
use clap::{crate_version, App, Arg};
use serde::Deserialize;
use std::io::Read;
use toml;
use tree_sitter::{Node, Parser, Range};

mod config;
mod ffi;
mod kakoune;
mod log;
mod tree;

#[derive(Deserialize)]
struct Request {
    op: Op,
    param: String,
    filetype: String,
    selections_desc: String,
    content: String,
}

#[derive(Deserialize)]
enum Op {
    NodeSExp,
    SelectChildren,
    SelectFirstChild,
    SelectKind,
    SelectNextNode,
    SelectParentKind,
    SelectParentNode,
    SelectPrevNode,
}

fn main() {
    let matches = cli();

    let verbosity = matches.occurrences_of("v") as u8;
    log::init_global_logger(verbosity);

    if let Some(filetype) = matches.value_of("do-you-understand") {
        let language = ffi::filetype_to_language(filetype);
        std::process::exit(if language.is_some() { 0 } else { 1 });
    }

    let config = if let Some(config_path) = matches.value_of("config") {
        Config::load(config_path).unwrap()
    } else {
        Config::default()
    };

    let mut request = String::new();
    std::io::stdin().read_to_string(&mut request).unwrap();
    let request: Request = toml::from_str(&request).unwrap();
    let response = handle_request(&config, &request);
    println!("{}", response);
}

fn cli() -> clap::ArgMatches<'static> {
    App::new("kak-tree")
        .version(crate_version!())
        .author("Ruslan Prokopchuk <fer.obbee@gmail.com>")
        .about("Structural selections for Kakoune")
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .value_name("FILE")
                .help("Read config from FILE")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("do-you-understand")
                .long("do-you-understand")
                .value_name("FILETYPE")
                .help("Exit with 0 if FILETYPE is supported, non-zero otherwise")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("v")
                .short("v")
                .multiple(true)
                .help("Sets the level of verbosity"),
        )
        .get_matches()
}

fn handle_request(config: &Config, request: &Request) -> String {
    let mut parser = Parser::new();
    let language = ffi::filetype_to_language(&request.filetype).unwrap();
    parser.set_language(language).unwrap();
    let tree = parser.parse_str(&request.content, None).unwrap();
    let buffer = request
        .content
        .split('\n')
        .map(|s| format!("{}\n", s))
        .collect::<Vec<_>>();
    let ranges = kakoune::selections_desc_to_ranges(&buffer, &request.selections_desc);
    let mut new_ranges = Vec::new();
    let filetype_config = config.get_filetype_config(&request.filetype);
    match &request.op {
        Op::SelectParentNode => {
            for range in &ranges {
                let mut node = tree::shrink_to_range(tree.root_node(), range);
                if node.range().start_byte == range.start_byte
                    && node.range().end_byte >= range.end_byte - 1
                {
                    node = node.parent().unwrap_or(node)
                }
                let node = traverse_up_to_node_which_matters(filetype_config, node);
                new_ranges.push(node.range());
            }
            kakoune::select_ranges(&buffer, &new_ranges)
        }
        Op::SelectNextNode => {
            for range in &ranges {
                let node = tree::shrink_to_range(tree.root_node(), range);
                let node = traverse_up_to_node_which_matters(filetype_config, node);
                if let Some(node) = node.next_named_sibling() {
                    new_ranges.push(node.range());
                } else {
                    new_ranges.push(node.range());
                }
            }
            kakoune::select_ranges(&buffer, &new_ranges)
        }
        Op::SelectPrevNode => {
            for range in &ranges {
                let node = tree::shrink_to_range(tree.root_node(), range);
                let node = traverse_up_to_node_which_matters(filetype_config, node);
                if let Some(node) = node.prev_named_sibling() {
                    new_ranges.push(node.range());
                } else {
                    new_ranges.push(node.range());
                }
            }
            kakoune::select_ranges(&buffer, &new_ranges)
        }
        Op::SelectFirstChild => {
            'outer: for range in &ranges {
                let node = tree::shrink_to_range(tree.root_node(), range);
                let node = traverse_up_to_node_which_matters(filetype_config, node);
                for child in tree::named_children(&node) {
                    if filetype_config.is_node_visible(child) {
                        new_ranges.push(child.range());
                        continue 'outer;
                    }
                }
                new_ranges.push(node.range());
            }
            kakoune::select_ranges(&buffer, &new_ranges)
        }
        Op::SelectChildren => {
            for range in &ranges {
                let node = tree::shrink_to_range(tree.root_node(), range);
                for child in tree::named_children(&node) {
                    if filetype_config.is_node_visible(child) {
                        new_ranges.push(child.range());
                    }
                }
            }
            kakoune::select_ranges(&buffer, &new_ranges)
        }
        Op::NodeSExp => {
            let node = tree::shrink_to_range(tree.root_node(), &ranges[0]);
            format!("info '{}'", node.to_sexp())
        }
        Op::SelectKind => {
            let kinds = filetype_config.resolve_alias(&request.param);
            for range in &ranges {
                for node in tree::nodes_in_range(tree.root_node(), range) {
                    select_nodes(&node, &kinds, &mut new_ranges);
                }
            }
            kakoune::select_ranges(&buffer, &new_ranges)
        }
        Op::SelectParentKind => {
            let kinds = filetype_config.resolve_alias(&request.param);
            for range in &ranges {
                let mut cursor = Some(tree::shrink_to_range(tree.root_node(), range));
                while let Some(node) = cursor {
                    if kinds.iter().any(|kind| kind == node.kind()) {
                        new_ranges.push(node.range());
                        break;
                    }
                    cursor = node.parent();
                }
            }
            kakoune::select_ranges(&buffer, &new_ranges)
        }
    }
}

fn select_nodes(node: &Node, kinds: &[String], new_ranges: &mut Vec<Range>) {
    if kinds.iter().any(|kind| kind == node.kind()) {
        new_ranges.push(node.range());
    } else {
        for child in tree::named_children(&node) {
            if kinds.iter().any(|kind| kind == child.kind()) {
                new_ranges.push(child.range());
            } else {
                select_nodes(&child, kinds, new_ranges);
            }
        }
    }
}

fn traverse_up_to_node_which_matters<'a>(
    filetype_config: &FiletypeConfig,
    current_node: Node<'a>,
) -> Node<'a> {
    let mut node = current_node;
    while !(node.is_named() && filetype_config.is_node_visible(node)) && node.parent().is_some() {
        node = node.parent().unwrap();
    }
    node
}
