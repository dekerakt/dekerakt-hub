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
use message::Message;

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

            let send = sink.send_all(stream.then(move |msg| {
                match msg {
                    Ok(Ok(msg)) => {
                        let response = handler.handle(msg.clone());

                        info!("addr" => format!("{}", addr);
                            "Handled a message: {:?}", msg);

                        Ok(response)
                    }

                    Ok(Err(e)) => {
                        let response = Message::Error {
                            description: format!("{}", e)
                        };

                        error!("Handled a bad message";
                            "error" => format!("{}", e),
                            "addr" => format!("{}", addr));

                        Ok(response)
                    }

                    Err(e) => Err(e)
                }
            }));


            handle.spawn(send.and_then(|_| Ok(())).or_else(move |e| {
                error!("Connection error";
                    "description" => e.description(),
                    "kind" => format!("{:?}", e.kind()),
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

