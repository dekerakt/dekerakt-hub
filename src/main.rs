extern crate futures;

extern crate tokio_service;
extern crate tokio_proto;
extern crate tokio_core;

#[macro_use(o, slog_log, slog_trace, slog_debug, slog_info, slog_warn,
            slog_error)]
extern crate slog;
#[macro_use]
extern crate slog_scope;
extern crate slog_term;

mod messages;
mod logger;

use std::env;
use std::{io, str};

use tokio_proto::pipeline::ServerProto;
use tokio_proto::TcpServer;

use tokio_service::Service;
use tokio_core::io::{Io, Framed, Codec, EasyBuf};
use futures::future;


struct LineProto;

impl<T: Io + 'static> ServerProto<T> for LineProto {
    type Request = String;
    type Response = String;
    type Error = io::Error;
    type Transport = Framed<T, LineCodec>;
    type BindTransport = io::Result<Framed<T, LineCodec>>;

    fn bind_transport(&self, io: T) -> io::Result<Framed<T, LineCodec>> {
        Ok(io.framed(LineCodec))
    }
}

struct LineCodec;

impl Codec for LineCodec {
    type In = String;
    type Out = String;

    fn decode(&mut self, buf: &mut EasyBuf) -> io::Result<Option<String>> {
        if let Some(pos) = buf.as_ref().iter().position(|&b| b == b'\n') {
            let line = buf.drain_to(pos);
            buf.drain_to(1);

            return match str::from_utf8(line.as_ref()) {
                Ok(v) => Ok(Some(v.to_string())),
                Err(_) =>
                    Err(io::Error::new(io::ErrorKind::Other, "invalid string"))
            };
        }

        Ok(None)
    }

    fn encode(&mut self, msg: String, buf: &mut Vec<u8>) -> io::Result<()> {
        for &byte in msg.as_bytes() {
            buf.push(byte);
        }

        buf.push(b'\n');
        Ok(())
    }
}

struct EchoService;

impl Service for EchoService {
    type Request = String;
    type Response = String;
    type Error = io::Error;
    type Future = future::Ok<String, io::Error>;

    fn call(&mut self, req: String) -> Self::Future {
        debug!("Got {:?} from somebody", req);
        future::ok(req)
    }
}

fn main() {
    logger::init();
    info!("Started app"; "version" => env!("CARGO_PKG_VERSION"));

    let addr = env::args().nth(1).unwrap_or("127.0.0.1:8080".to_string());
    let addr = addr.parse().unwrap();

    TcpServer::new(LineProto, addr)
        .serve(|| Ok(EchoService));
}
