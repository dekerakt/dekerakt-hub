extern crate mio;
extern crate slab;
extern crate bytes;

#[macro_use]
extern crate error_chain;

#[macro_use]
extern crate slog;
extern crate slog_term;
extern crate slog_async;

mod error;
mod protocol;
mod codec;
mod server;

use slog::{Logger, Level};
use server::Server;

fn main() {
    let addr = "127.0.0.1:8080".parse().unwrap();
    let logger = root_logger(Level::Trace);

    let server = match Server::with_logger(logger.clone(), &addr) {
        Ok(v) => v,
        Err(e) => {
            crit!(logger, "Creating server failed: {}", e);
            return;
        }
    };

    if let Err(e) = server.run() {
        crit!(logger, "Server crashed: {}", e);
    }
}

fn root_logger(level: Level) -> Logger {
    use slog::Drain;

    let decorator = slog_term::TermDecorator::new().stderr().build();
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();
    let drain = slog::LevelFilter::new(drain, level).fuse();

    Logger::root(std::sync::Arc::new(drain), o!())
}
