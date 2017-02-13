use mio::tcp::*;

use store::Store;
use handler::Handler;
use codec::Codec;
use message::Message;

const SERVTOKEN: mio::Token = mio::Token(0);

struct Server {
    server: TcpListener,
    connections: Slab<Connection>
}

impl Server {
    fn new(address: SocketAddr) -> Server {
        let server = TcpListener::bind(&address).unwrap();
        let mut event_loop = mio::EventLoop::new().unwrap();
        event_loop.register(&server, SERVER);

        let slab = Slab::new_starting_at(mio::Token(1), 8192);

        event_loop.run(&mut Server {
            server: server,
            connections: slab
        });
    }
}

impl mio::Handler for Server {
    type Timeout = ();
    type Message = ();

    fn ready(&mut self, event_loop: &mut mio::EventLoop<Server>, token: mio::Token, events: mio::EventSet) {
        match token {
            SERVER => {
                assert!(events.is_readable());

                match self.server.accept() {
                    Ok(Some(socket)) => {
                        let token = self.connections
                            .insert_with(|token| Connection::new(socket, token))
                            .unwrap();

                        event_loop.register_opt(
                            &self.connections[token].socket,
                            token,
                            mio::EventSet::readable(),
                            mio::PollOpt::edge() | mio::PollOpt::oneshot()).unwrap();
                    }
                    Ok(None) => {}
                    Err(e) => {
                        error!("accept errored: {}", e);
                        event_loop.shutdown();
                    }
                }
            }
            _ => {
                self.connections[token].read(event_loop, events);

                if self.connections[token].is_closed() {
                    self.connections.remove(token);
                }
            }
        }
    }
}

struct Connection {
    socket: TcpStream,
    token: mio::Token,
    state: State,
    side: ConnectionSide
}

enum State {
    Begin,
    AuthClient,
    InitialData,
    Connected
}
