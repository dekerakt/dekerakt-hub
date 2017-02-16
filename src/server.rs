use mio::*;
use mio::tcp::*;

use std::net::SocketAddr;
use std::cell::RefCell;

use linked_hash_map::LinkedHashMap;

use store::Store;
use message::*;

const SERVTOKEN: Token = Token(0);

type Conns = LinkedHashMap<Token, Connection>;

pub struct Server {
    server: TcpListener,
    connections: Conns,
    events: Events,
    poll: Poll,
    store: Store,
}

impl Server {
    pub fn new(address: SocketAddr) -> Server {
        let server = TcpListener::bind(&address).unwrap();
        let poll = Poll::new().unwrap();
        poll.register(&server, SERVTOKEN, Ready::readable(),
                      PollOpt::edge()).unwrap();
        let mut events = Events::with_capacity(8192);
        let mut conns = Conns::new();
        let mut store = Store::new();

        Server {
            server: server,
            connections: conns,
            events: events,
            poll: poll,
            store: store,
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
                                let conn = Connection::new(RefCell::new(*self), socket, token);
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
    side: ConnectionSide,
    server: RefCell<Server>
}

impl Connection {
    pub fn new(server: RefCell<Server>, socket: TcpStream, token: Token) -> Connection {
        Connection {
            socket: socket,
            token: token,
            state: State::Begin,
            side: ConnectionSide::None,
            server: server
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

    pub fn handle(&mut self, msg: Message) {
        match msg {
            Message::Error { description } => {}
            Message::AuthClient {
                user, password, connection_mode, connection_side, ping_interval
            } => {
                match self.state {
                    State::AuthClient => {}
                    _ => {
                        return;
                    }
                }
                if let Some(vertex) = self.server.borrow_mut().store.find_vertex_with_user_mut(user) {
                    let doubling = match self.side {
                        ConnectionSide::OC => {
                            true
                        }
                        ConnectionSide::External => {
                            match vertex.external {
                                Some(..) => true,
                                None => false
                            }
                        }
                        _ => {
                            unimplemented!();
                        }
                    };
                    if doubling {
                        self.send(Message::AuthServer {
                            auth_result: AuthResult::VertexInUse,
                            display_message: "".to_string()
                        });
                        self.close();
                    } else {
                        if vertex.password == password {
                            vertex.external = Some(self.token);
                            self.send(Message::AuthServer {
                                auth_result: AuthResult::Authenticated,
                                display_message: "".to_string()
                            });
                        } else {
                            self.send(Message::AuthServer {
                                auth_result: AuthResult::BadCredentials,
                                display_message: "".to_string()
                            });
                            self.close();
                        }
                    }
                } else {
                    match self.side {
                        ConnectionSide::External => {
                            self.send(Message::AuthServer {
                                auth_result: AuthResult::BadCredentials,
                                display_message: "".to_string()
                            });
                            self.close();
                        }
                        ConnectionSide::OC => {
                            match connection_mode {
                                ConnectionMode::GpuKbd => {
                                    self.send(Message::AuthServer {
                                        auth_result: AuthResult::Authenticated,
                                        display_message: "".to_string()
                                    });
                                    self.state = State::InitialData(user, password, connection_mode);
                                }
                                _ => {
                                    self.send(Message::AuthServer {
                                        auth_result: AuthResult::UnsupportedMode,
                                        display_message: "".to_string()
                                    });
                                    self.close();
                                }
                            }
                        }
                        _ => {
                            unimplemented!();
                        }
                    }
                }
            }
            _ => {
                unimplemented!();
            }
        }
    }

    pub fn close(&mut self) {
        self.state = State::Closed;
    }

    pub fn send(&mut self, msg: Message) {
        unimplemented!();
    }
}

pub enum State {
    Begin,
    AuthClient,
    InitialData(String, String, ConnectionMode),
    Connected,
    Closed
}
