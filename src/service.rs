use std::io;
use std::net::SocketAddr;
use std::rc::Rc;
use std::cell::RefCell;
use std::ops::Drop;

use tokio_service::Service as ServiceTrait;
use futures::{future, Future};

use store::Store;

pub struct Service {
    addr: SocketAddr,
    store: Rc<RefCell<Store>>
}

impl Service {
    pub fn new(addr: SocketAddr, store: Rc<RefCell<Store>>) -> Service {
        *store.borrow_mut() += 1;
        info!("User connected"; "addr" => format!("{}", addr));

        Service {
            addr: addr,
            store: store
        }
    }
}

impl Drop for Service {
    fn drop(&mut self) {
        info!("User disconnected"; "addr" => format!("{}", self.addr));
        *self.store.borrow_mut() -= 1;
    }
}

impl ServiceTrait for Service {
    type Request = String;
    type Response = String;
    type Error = io::Error;
    type Future = future::Ok<String, io::Error>;

    fn call(&mut self, req: String) -> Self::Future {
        info!("Got a message"; "addr" => format!("{}", self.addr),
                               "text" => req);
        future::ok(req)
    }
}

