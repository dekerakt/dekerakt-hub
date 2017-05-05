use std::io::{self, Read, Write};
use std::net::SocketAddr;

use bytes::{BufMut, BytesMut};

use mio::{Token, Poll, Event, Events, PollOpt, Ready};
use mio::net::{TcpListener, TcpStream};

use slog::Logger;
use slab::Slab;

use protocol::{encode_to_bytesmut, decode_from_bytesmut, Message};
use error::{Error, ErrorKind, Result};

const SERVER_TOKEN: Token = Token(::std::usize::MAX - 10);

const EVENTS_CAPACITY: usize = 1024;
const CONNECTIONS_CAPACITY: usize = 8192;
const CONNECTION_READ_BUF_CAPACITY: usize = 1;
const CONNECTION_READ_CHUNK_SIZE: usize = 4096;
const CONNECTION_WRITE_BUF_CAPACITY: usize = 4096;

pub struct Server {
    logger: Logger,
    socket: TcpListener,
    poll: Poll,

    addr: SocketAddr,
    connections: Slab<Client, Token>,
}

impl Server {
    pub fn with_logger(logger: Logger, addr: &SocketAddr) -> Result<Server> {
        Ok(Server {
               logger: logger,
               socket: TcpListener::bind(addr)?,
               poll: Poll::new()?,

               addr: addr.clone(),
               connections: Slab::with_capacity(CONNECTIONS_CAPACITY),
           })
    }

    pub fn run(mut self) -> Result<()> {
        self.register()?;

        let mut events = Events::with_capacity(EVENTS_CAPACITY);
        info!(self.logger, "Listening on {}", self.addr);

        loop {
            let amt = self.poll.poll(&mut events, None)?;

            trace!(self.logger, "Handling {} events", amt);

            for event in events.iter() {
                trace!(self.logger, " - {:?}", event);
                self.handle_event(event)?;
            }
        }
    }

    fn handle_event(&mut self, event: Event) -> Result<()> {
        match event.token() {
            SERVER_TOKEN => self.accept(),
            _ => self.handle_client(event),
        }
    }

    fn accept(&mut self) -> Result<()> {
        let (socket, addr) = match self.socket.accept() {
            Ok(v) => v,
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => return Ok(()),
            Err(e) => return Err(e.into()),
        };

        let entry = match self.connections.vacant_entry() {
            Some(v) => v,
            None => {
                warn!(self.logger,
                      "No more space in the slab; reallocation unimplemented");
                return Ok(()); // TODO: reallocate slab
            }
        };

        let token = entry.index();
        let logger = self.logger
            .new(o!("addr" => format!("{}", addr),
                                        "token" => format!("{:?}", token)));

        let client = Client::with_logger(logger, socket, token);
        client.register(&self.poll)?;

        info!(client.logger, "Connected; waiting for a handshake");

        entry.insert(client);

        Ok(())
    }

    fn handle_client(&mut self, event: Event) -> Result<()> {
        let mut client = match self.connections.entry(event.token()) {
            Some(v) => v,
            None => return Err(ErrorKind::InvalidToken(event.token()).into()),
        };

        let readiness = event.readiness();

        if readiness.is_readable() {
            // Important: first read, then write
            client.get_mut().readable()?;
        }

        if readiness.is_writable() {
            client.get_mut().writable()?;
        }

        let state = client.get().state;

        match state {
            ClientState::Error if client.get().write_buf.is_empty() => {
                client.get().deregister(&self.poll)?;
                info!(client.get().logger, "Closed");
                client.remove();
            }

            ClientState::Dead => {
                client.get().deregister(&self.poll)?;
                info!(client.get().logger, "Closed");
                client.remove();
            }

            _ => {}
        }

        Ok(())
    }

    fn register(&self) -> Result<()> {
        trace!(self.logger, "Registering server socket";
               "token" => format!("{:?}", SERVER_TOKEN),
               "interest" => format!("{:?}", Ready::readable()),
               "opts" => format!("{:?}", PollOpt::edge()));

        self.poll
            .register(&self.socket,
                      SERVER_TOKEN,
                      Ready::readable(),
                      PollOpt::edge())?;

        Ok(())
    }
}

#[derive(Copy, Clone)]
enum ClientState {
    Ok,
    Dead,
    Error,
}

struct Client {
    logger: Logger,
    socket: TcpStream,
    token: Token,

    read_buf: BytesMut,
    write_buf: BytesMut,
    state: ClientState,
}

impl Client {
    fn with_logger(logger: Logger, socket: TcpStream, token: Token) -> Client {
        Client {
            logger: logger,
            socket: socket,
            token: token,

            read_buf: BytesMut::with_capacity(CONNECTION_READ_BUF_CAPACITY),
            write_buf: BytesMut::with_capacity(CONNECTION_WRITE_BUF_CAPACITY),
            state: ClientState::Ok,
        }
    }


    fn readable(&mut self) -> Result<()> {
        let mut chunk = [0; CONNECTION_READ_CHUNK_SIZE];
        let start_len = self.read_buf.len();

        loop {
            match self.socket.read(&mut chunk[..]) {
                Ok(0) => {
                    trace!(self.logger, "Read 0 bytes; closing");
                    self.state = ClientState::Dead;
                    return Ok(());
                }

                Ok(amt) => {
                    if amt > self.read_buf.remaining_mut() {
                        warn!(self.logger, "Read buffer overflowed; reallocation required");
                        self.read_buf.reserve(amt);
                    }

                    self.read_buf.put_slice(&chunk[..amt]);
                }

                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => break,
                Err(e) => return Err(e.into()),
            }
        }

        let len = self.read_buf.len() - start_len;

        trace!(self.logger, "Read {} bytes", len);
        self.parse_messages();

        Ok(())
    }

    fn parse_messages(&mut self) {
        let mut error = None;
        loop {
            let msg: Message = match decode_from_bytesmut(&mut self.read_buf) {
                Ok(Some(v)) => v,
                Ok(None) => break,
                Err(e) => {
                    error = Some(e);
                    break;
                }
            };

            self.handle_message(msg);
        }

        if let Some(e) = error {
            self.error(e);
        }
    }

    fn handle_message(&mut self, msg: Message) {
        info!(self.logger, "IN  {} message", msg);
        error!(self.logger, "Messages handling unimplemented; ignoring");
    }

    fn send_message(&mut self, msg: Message) {
        info!(self.logger, "OUT {} message", msg);
        encode_to_bytesmut(&mut self.write_buf, msg);
    }

    fn error(&mut self, e: Error) {
        warn!(self.logger, "Closing with error[{}]", e);
        self.send_message(Message::Error(e));
        self.state = ClientState::Error;
    }

    fn writable(&mut self) -> Result<()> {
        if self.write_buf.is_empty() {
            return Ok(());
        }

        let start_len = self.write_buf.len();

        while self.write_buf.len() > 0 {
            match self.socket.write(&self.write_buf) {
                Ok(amt) => self.write_buf.split_to(amt),
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => break,
                Err(e) => return Err(e.into()),
            };
        }

        let len = start_len - self.write_buf.len();

        trace!(self.logger, "Wrote {} bytes", len);

        Ok(())
    }

    fn register(&self, poll: &Poll) -> Result<()> {
        trace!(self.logger, "Registering client socket";
               "token" => format!("{:?}", self.token),
               "interest" => format!("{:?}", Ready::readable() | Ready::writable()),
               "opts" => format!("{:?}", PollOpt::edge()));

        poll.register(&self.socket,
                      self.token,
                      Ready::readable() | Ready::writable(),
                      PollOpt::edge())?;

        Ok(())
    }

    fn deregister(&self, poll: &Poll) -> Result<()> {
        trace!(self.logger, "Deregistering client socket");

        poll.deregister(&self.socket)?;

        Ok(())
    }
}
