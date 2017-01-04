use std::net::SocketAddr;
use std::rc::Rc;
use std::cell::RefCell;

use store::Store;

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

    pub fn handle(&mut self, msg: String) -> String {
        msg.chars().rev().collect()
    }
}
