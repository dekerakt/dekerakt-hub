extern crate futures;
extern crate tokio_proto;
extern crate tokio_core;

use std::env;
use std::net::SocketAddr;

use futures::Future;
use futures::stream::Stream;
use tokio_core::io::{copy, Io};
use tokio_core::net::TcpListener;
use tokio_core::reactor::Core;

fn main() {
    let addr = env::args().nth(1).unwrap_or("127.0.0.1:8080".to_string());
    let addr = addr.parse().unwrap();

    let mut core = Core::new().unwrap();
    let handle = core.handle();

    let socket = TcpListener::bind(&addr, &handle).unwrap();
    println!("Listening on {}", addr);

    let done = socket.incoming().for_each(|(socket, addr)| {
        let (reader, writer) = socket.split();
        let amt = copy(reader, writer);

        let msg = amt.then(move |result| {
            match result {
                Ok(amt) => println!("wrote {} bytes to {}", amt, addr),
                Err(e) => println!("error on {}: {}", addr, e),
            }

            Ok(())
        });

        handle.spawn(msg);

        Ok(())
    });

    core.run(done).unwrap();
}
