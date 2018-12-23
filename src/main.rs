#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate slog;
#[macro_use]
extern crate slog_scope;

use clap::{crate_version, App, Arg};
use serde::{Deserialize, Serialize};
use sloggers::terminal::{Destination, TerminalLoggerBuilder};
use sloggers::types::Severity;
use sloggers::Build;
use std::io::{Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use toml;
use tree_sitter::{Language, Parser};

extern "C" {
    fn tree_sitter_rust() -> Language;
}

fn main() {
    let matches = App::new("kak-tree")
        .version(crate_version!())
        .author("Ruslan Prokopchuk <fer.obbee@gmail.com>")
        .about("Structural editing for Kakoune")
        .arg(
            Arg::with_name("session")
                .short("s")
                .long("session")
                .value_name("SESSION")
                .help("Unix socket path to listen to requests")
                .takes_value(true)
                .required(true),
        )
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

    let socket_path = matches.value_of("session").unwrap();
    let listener = UnixListener::bind(&socket_path).unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let mut request = String::new();
                match stream.read_to_string(&mut request) {
                    Ok(_) => {
                        let request: Request = toml::from_str(&request).unwrap();
                        let response = handle_request(request);
                        stream.write_all(&toml::to_vec(&response).unwrap()).unwrap();
                    }
                    Err(e) => {
                        error!("Failed to read from TCP stream: {}", e);
                    }
                }
            }
            Err(e) => {
                error!("Failed to accept connection: {}", e);
            }
        }
    }
}

fn handle_request(request: Request) -> Response {
    // TODO handle ops variety
    // TODO handle langugae variety
    // TODO move outside init of shared stuff
    // TODO leverage incremental parsing
    let mut parser = Parser::new();
    let language = unsafe { tree_sitter_rust() };
    parser.set_language(language).unwrap();
    let tree = parser.parse_str(&request.content, None).unwrap();
    let mut cursor = tree.walk();
    cursor.goto_first_child_for_index(*request.meta.cursors_byte_offset.first().unwrap());
    debug!("{:?}", cursor.node());
    Response {}
}

#[derive(Deserialize)]
struct Request {
    meta: Meta,
    op: Op,
    content: String,
}

#[derive(Serialize)]
struct Response {}

#[derive(Deserialize)]
struct Meta {
    filetype: String,
    cursors_byte_offset: Vec<usize>,
    selections_desc: String,
}

#[derive(Deserialize)]
enum Op {
    ExpressionRanges,
}
