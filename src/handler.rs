use std::io;
use std::net::SocketAddr;
use std::rc::Rc;
use std::cell::RefCell;

use store::Store;
use codec::Error;

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

    pub fn handle(&mut self, msg: Result<String, Error>) -> String {
        match msg {
            Ok(msg) => msg.chars().rev().collect(),
            Err(e) => format!("{}", e)
        }
    }
}
