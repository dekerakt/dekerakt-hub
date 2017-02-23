use mio::*;
use mio::tcp::*;

use std::net::SocketAddr;
use std::rc::Rc;
use std::cell::RefCell;
use std::ops::{Deref, DerefMut};

use linked_hash_map::LinkedHashMap;

use message::*;
use graphics::*;

const SERVTOKEN: Token = Token(0);

pub struct Conns(LinkedHashMap<Token, Connection>);

impl Conns {
    pub fn new() -> Conns {
        Conns(LinkedHashMap::new())
    }

    pub fn find_vertex_with_user(&self, user: &String) -> Option<(Token, &ClientData)> {
        for (token, conn) in self.0.iter() {
            match conn.vertex {
                Client::OC(ref v) => {
                    if v.user == *user {
                        return Some((*token, v));
                    }
                },
                _ => {}
            }
        }
        None
    }

    pub fn find_vertex_with_user_mut(&mut self, user: String) -> Option<(Token, &mut ClientData)> {
        for (token, conn) in self.0.iter_mut() {
            match conn.vertex {
                Client::OC(ref mut v) => {
                    if v.user == *user {
                        return Some((*token, v));
                    }
                },
                _ => {}
            }
        }
        None
    }

    pub fn find_external_for(&self, token: Token) -> Option<Token> {
        for (external_token, conn) in self.0.iter() {
            match conn.vertex {
                Client::External(t) if t == token => {
                    return Some(*external_token);
                }
                _ => {}
            }
        }
        None
    }
}

impl Deref for Conns {
    type Target = LinkedHashMap<Token, Connection>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Conns {
    fn deref_mut(&mut self) -> &mut LinkedHashMap<Token, Connection> {
        &mut self.0
    }
}

pub struct ClientData {
    pub palette: [Color; 16],
    pub mode: ConnectionMode,
    pub user: String,
    pub password: String,
    pub screen_state: ScreenState,
    pub precise_mode: PreciseMode,
    pub canvas: Canvas,
}

pub struct Server {
    server: TcpListener,
    connections: Rc<RefCell<Conns>>,
    events: Events,
    poll: Poll
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
            connections: Rc::new(RefCell::new(conns)),
            events: events,
            poll: poll
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
                                                              .borrow()
                                                              .back()
                                                              .unwrap()
                                                              .0) + 1);
                                }
                                let conn = Connection::new(
                                    self.connections.clone(), socket, token);
                                {
                                    self.connections.borrow_mut().insert(token, conn);
                                }

                                self.poll.register(
                                    &self.connections.borrow()[&event.token()].socket,
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
                            let c = self.connections.borrow_mut()
                                .get_mut(&event.token())
                                .unwrap()
                                .ready(event.kind());
                        }

                        if self.connections.borrow()[&event.token()].is_closed() {
                            self.connections.borrow_mut().remove(&event.token());
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
    connections: Rc<RefCell<Conns>>,
    vertex: Client
}

impl Connection {
    pub fn new(connections: Rc<RefCell<Conns>>, socket: TcpStream, token: Token) -> Connection {
        Connection {
            socket: socket,
            token: token,
            state: State::Begin,
            side: ConnectionSide::None,
            connections: connections,
            vertex: Client::None
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
                if let Some((oc_token, vertex)) = self.connections.borrow_mut().find_vertex_with_user_mut(user) {
                    match self.connections.borrow().find_external_for(self.token) {
                        Some(_) => {
                            self.send(Message::AuthServer {
                                auth_result: AuthResult::VertexInUse,
                                display_message: "".to_string()
                            });
                            self.close();
                        }
                        None => {
                            if vertex.password == password {
                                self.vertex = Client::External(oc_token);
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

pub enum Client {
    OC(ClientData),
    External(Token),
    None
}
