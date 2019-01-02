use sloggers::terminal::{Destination, TerminalLoggerBuilder};
use sloggers::types::Severity;
use sloggers::Build;

pub fn init_global_logger(verbosity: u8) {
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
}
