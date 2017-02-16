use mio::*;
use mio::tcp::*;
use std::net::SocketAddr;
use std::io::Error;

use linked_hash_map::LinkedHashMap;

use store::Store;
use handler::Handler;
use message::{Message, ConnectionSide};

const SERVTOKEN: Token = Token(0);

type Conns = LinkedHashMap<Token, Connection>;

pub struct Server {
    server: TcpListener,
    connections: Conns,
    events: Events,
    poll: Poll,
}

impl Server {
    pub fn new(address: SocketAddr) -> Server {
        let server = TcpListener::bind(&address).unwrap();
        let poll = Poll::new().unwrap();
        poll.register(&server, SERVTOKEN, Ready::readable(),
                      PollOpt::edge()).unwrap();
        let mut events = Events::with_capacity(8192);

        let mut conns = Conns::new();

        Server {
            server: server,
            connections: conns,
            events: events,
            poll: poll,
        }
    }

    pub fn run(&mut self) {
        'event_loop: loop {
            self.poll.poll(&mut self.events, None).unwrap();

            for event in self.events.iter() {
                match event.token() {
                    SERVTOKEN => {
                        assert!(event.kind().is_readable());

                        match self.server.accept() {
                            Ok((socket, addr)) => {
                                let token: Token;
                                {
                                    token = Token(usize::from(*self.connections
                                                              .back()
                                                              .unwrap()
                                                              .0) + 1);
                                }
                                let conn = Connection::new(socket, token);
                                {
                                    self.connections.insert(token, conn);
                                }

                                self.poll.register(
                                    &self.connections[&event.token()].socket,
                                    token,
                                    Ready::readable(),
                                    PollOpt::edge()).unwrap();
                            }
                            Err(e) => {
                                error!("accept errored: {}", e);
                                break 'event_loop;
                            }
                        }
                    }
                    _ => {
                        {
                            let c = self.connections.get_mut(&event.token())
                                .unwrap();
                            c.ready(event.kind());
                        }

                        if self.connections[&event.token()].is_closed() {
                            self.connections.remove(&event.token());
                        }
                    }
                }
            }
        }
    }
}

pub struct Connection {
    socket: TcpStream,
    token: Token,
    state: State,
    side: ConnectionSide
}

impl Connection {
    pub fn new(socket: TcpStream, token: Token) -> Connection {
        Connection {
            socket: socket,
            token: token,
            state: State::Begin,
            side: ConnectionSide::None
        }
    }

   pub fn ready(&mut self, kind: Ready) {
        unimplemented!();
    }

    pub fn is_closed(&self) -> bool {
        match self.state {
            State::Closed => true,
            _ => false
        }
    }
}

pub enum State {
    Begin,
    AuthClient,
    InitialData,
    Connected,
    Closed
}
