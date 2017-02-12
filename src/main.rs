#[macro_use(o, slog_log, slog_trace, slog_debug, slog_info, slog_warn,
    slog_error, slog_crit)]
extern crate slog;
#[macro_use]
extern crate slog_scope;
extern crate slog_term;

extern crate unicode_width;
extern crate argparse;

extern crate mio;

mod graphics;
mod message;
mod logger;
mod codec;
mod handler;
mod store;
mod server;

use std::process::exit;
use std::net::SocketAddr;

use slog::Level;
use argparse::{ArgumentParser, StoreConst, Store};

use server::Server;


fn parse_arguments() -> (slog::Level, SocketAddr) {
        let mut level = Level::Info;
        let mut addr = "127.0.0.1:8080".to_string();

    {
        let mut ap = ArgumentParser::new();

        ap.refer(&mut level)
            .add_option(&["-v", "--verbose"], StoreConst(Level::Debug),
            "be verbose");

        ap.refer(&mut addr)
            .add_argument("address", Store,
            "listener address");

        ap.parse_args_or_exit();
    }

    (level, addr.parse().unwrap_or_else(|e| {
        println!("{}", e);
        exit(2);
    }))
}


fn main() {
    let (level, addr) = parse_arguments();
    logger::init(level);

    info!("Started app"; "version" => env!("CARGO_PKG_VERSION"));

    let mut server = Server::new(addr);

    match server.run() {
        Err(e) => crit!("{}", e),
        _ => {}
    }
}
