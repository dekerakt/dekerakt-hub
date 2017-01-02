use {slog, slog_scope, slog_term};
use slog::{Level, Logger, DrainExt, level_filter};

pub fn init() {
    let drain = slog_term::streamer().stderr().build().fuse();
    let drain = level_filter(Level::Debug, drain);
    slog_scope::set_global_logger(Logger::root(drain, o![]));
}

