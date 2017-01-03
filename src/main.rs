extern crate futures;

extern crate tokio_service;
extern crate tokio_proto;
extern crate tokio_core;

#[macro_use(o, slog_log, slog_trace, slog_debug, slog_info, slog_warn,
            slog_error, slog_crit)]
extern crate slog;
#[macro_use]
extern crate slog_scope;
extern crate slog_term;

extern crate unicode_width;

mod graphics;
mod message;
mod logger;
mod codec;
mod proto;
mod service;
mod store;
mod server;

use std::env;
use server::Server;

fn main() {
    logger::init();
    info!("Started app"; "version" => env!("CARGO_PKG_VERSION"));

    let addr = env::args().nth(1).unwrap_or("127.0.0.1:8080".to_string());
    let addr = addr.parse().unwrap();

    let mut server = Server::new(addr);

    match server.run() {
        Err(e) => crit!("{}", e),
        _ => {}
    }
}
