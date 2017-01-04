use std::io;
use std::net::SocketAddr;
use std::rc::Rc;
use std::cell::RefCell;
use std::error::Error;

use tokio_core::reactor::Core;
use tokio_core::net::TcpListener;
use tokio_core::io::Io;

use futures::{Sink, Stream, Future};

use store::Store;
use handler::Handler;
use codec::Codec;

pub struct Server {
    pub addr: SocketAddr,
    pub store: Rc<RefCell<Store>>
}

impl Server {
    pub fn new(addr: SocketAddr) -> Server {
        Server {
            addr: addr,
            store: Rc::new(RefCell::new(0))
        }
    }

    pub fn run(&mut self) -> io::Result<()> {
        let mut core = Core::new()?;
        let handle = core.handle();

        let listener = TcpListener::bind(&self.addr, &handle)?;
        info!("Started listening"; "addr" => format!("{}", self.addr));

        let store = self.store.clone();

        let done = listener.incoming().for_each(move |(socket, addr)| {
            info!("New connection"; "addr" => format!("{}", addr));

            let framed = socket.framed(Codec);
            let (sink, stream) = framed.split();

            let store = store.clone();
            let mut handler = Handler::new(addr, store);

            let send = sink.send_all(stream.and_then(move |msg| {
                let response = handler.handle(msg.clone());
                info!("Handled request";
                    "request" => format!("{:?}", msg),
                    "response" => format!("{:?}", response),
                    "addr" => format!("{}", addr));

                Ok(response)
            }));

            handle.spawn(send.and_then(|_| Ok(())).or_else(move |e| {
                error!("Connection error";
                    "description" => e.description(),
                    "addr" => format!("{}", addr));
                Ok(())
            }).and_then(move |_| {
                info!("Connection lost"; "addr" => format!("{}", addr));
                Ok(())
            }));

            Ok(())
        });

        core.run(done)
    }
}

