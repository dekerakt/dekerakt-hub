use std::io;
use std::net::SocketAddr;
use std::rc::Rc;
use std::cell::RefCell;

use tokio_core::reactor::Core;
use tokio_core::net::TcpListener;
use tokio_service::NewService;
use tokio_proto::BindServer;

use futures::Stream;

use service::Service;
use proto::Proto;
use store::Store;

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

        let binder = Proto;

        let done = listener.incoming().for_each(move |(socket, addr)| {
            let store = self.store.clone();
            let service = Service::new(addr, store);

            binder.bind_server(&handle, socket, service);

            Ok(())
        });

        core.run(done)
    }
}

