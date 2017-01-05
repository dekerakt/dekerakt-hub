use {slog_scope, slog_term};
use slog::{Level, Logger, DrainExt, level_filter};

pub fn init(level: Level) {
    let drain = slog_term::streamer().stderr().build().fuse();
    let drain = level_filter(level, drain);
    slog_scope::set_global_logger(Logger::root(drain, o![]));
}

