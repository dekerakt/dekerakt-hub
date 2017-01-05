use std::net::SocketAddr;
use std::rc::Rc;
use std::cell::RefCell;

use store::Store;
use message::Message;

pub struct Handler {
    addr: SocketAddr,
    store: Rc<RefCell<Store>>
}

impl Handler {
    pub fn new(addr: SocketAddr, store: Rc<RefCell<Store>>) -> Handler {
        Handler {
            addr: addr,
            store: store
        }
    }

    pub fn handle(&mut self, msg: Message) -> Message {
        match msg {
            Message::Ping { ping } => Message::Pong { pong: ping },

            _ => Message::Error {
                description: "I'm a teapot!".to_string()
            }
        }
    }
}
