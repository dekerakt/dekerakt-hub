use std::fmt;
use number_prefix::{binary_prefix, Standalone, Prefixed};
use error::{Result, ErrorKind};

#[derive(Debug, Clone)]
pub enum Message {
    Error(String),
    CriticalError(String),
    Ping(u64),
    Pong(u64),

    ClientHandshake { username: String, password: String },
    ClientConnect { username: String, password: String },
    ClientDisconnect,
    ClientGoodbye,

    ServerHandshake(HandshakeStatus),
    ServerConnect(ConnectionStatus),
    ServerDisconnect(DisconnectionStatus),
    ServerGoodbye,

    PairPing(u64),
    PairPong(u64),
    PairText(String),
    PairBinary(Vec<u8>),
}

impl Message {
    pub fn opcode(&self) -> Opcode {
        match *self {
            Message::Error(..) => Opcode::Error,
            Message::CriticalError(..) => Opcode::CriticalError,
            Message::Ping(..) => Opcode::Ping,
            Message::Pong(..) => Opcode::Pong,

            Message::ClientHandshake { .. } => Opcode::ClientHandshake,
            Message::ClientConnect { .. } => Opcode::ClientConnect,
            Message::ClientDisconnect => Opcode::ClientDisconnect,
            Message::ClientGoodbye => Opcode::ClientGoodbye,

            Message::ServerHandshake(..) => Opcode::ServerHandshake,
            Message::ServerConnect(..) => Opcode::ServerConnect,
            Message::ServerDisconnect(..) => Opcode::ServerDisconnect,
            Message::ServerGoodbye => Opcode::ServerGoodbye,

            Message::PairPing(..) => Opcode::PairPing,
            Message::PairPong(..) => Opcode::PairPong,
            Message::PairText(..) => Opcode::PairText,
            Message::PairBinary(..) => Opcode::PairBinary,
        }
    }
}

impl fmt::Display for Message {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Message::Error(ref s) => write!(fmt, "error[{}]", s),
            Message::CriticalError(ref s) => write!(fmt, "critical-error[{}]", s),
            Message::Ping(t) => write!(fmt, "ping[{}]", t),
            Message::Pong(t) => write!(fmt, "pong[{}]", t),

            Message::ClientHandshake { ref username, .. } => {
                write!(fmt, "client-handshake[{}]", username)
            }

            Message::ClientConnect { ref username, .. } => {
                write!(fmt, "client-connect[{}]", username)
            }

            Message::ClientDisconnect => write!(fmt, "client-disconnect"),
            Message::ClientGoodbye => write!(fmt, "client-goodbye"),

            Message::ServerHandshake(s) => write!(fmt, "server-handshake[{}]", s),
            Message::ServerConnect(s) => write!(fmt, "server-connect[{}]", s),
            Message::ServerDisconnect(s) => write!(fmt, "server-disconnect[{}]", s),
            Message::ServerGoodbye => write!(fmt, "server-goodbye"),

            Message::PairPing(t) => write!(fmt, "pair-ping[{}]", t),
            Message::PairPong(t) => write!(fmt, "pair-pong[{}]", t),

            Message::PairText(ref d) => {
                match binary_prefix(d.len() as f32) {
                    Standalone(b) => write!(fmt, "pair-text[{}B]", b),
                    Prefixed(prefix, n) => write!(fmt, "pair-text[{}{}B]", n, prefix),
                }
            }

            Message::PairBinary(ref d) => {
                match binary_prefix(d.len() as f32) {
                    Standalone(b) => write!(fmt, "pair-binary[{} B]", b),
                    Prefixed(prefix, n) => write!(fmt, "pair-text[{:.0} {}B]", n, prefix),
                }
            }
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Opcode {
    Error = 0x00,
    CriticalError = 0x01,
    Ping = 0x02,
    Pong = 0x03,

    ClientHandshake = 0x40,
    ClientConnect = 0x41,
    ClientDisconnect = 0x42,
    ClientGoodbye = 0x43,

    ServerHandshake = 0x80,
    ServerConnect = 0x81,
    ServerDisconnect = 0x82,
    ServerGoodbye = 0x83,

    PairPing = 0xc0,
    PairPong = 0xc1,
    PairText = 0xc2,
    PairBinary = 0xc3,
}

impl Opcode {
    pub fn as_u8(&self) -> u8 {
        *self as u8
    }

    pub fn from_u8(c: u8) -> Result<Opcode> {
        match c {
            0x00 => Ok(Opcode::Error),
            0x01 => Ok(Opcode::CriticalError),
            0x02 => Ok(Opcode::Ping),
            0x03 => Ok(Opcode::Pong),

            0x40 => Ok(Opcode::ClientHandshake),
            0x41 => Ok(Opcode::ClientConnect),
            0x42 => Ok(Opcode::ClientDisconnect),
            0x43 => Ok(Opcode::ClientGoodbye),

            0x80 => Ok(Opcode::ServerHandshake),
            0x81 => Ok(Opcode::ServerConnect),
            0x82 => Ok(Opcode::ServerDisconnect),
            0x83 => Ok(Opcode::ServerGoodbye),

            0xc0 => Ok(Opcode::PairPing),
            0xc1 => Ok(Opcode::PairPong),
            0xc2 => Ok(Opcode::PairText),
            0xc3 => Ok(Opcode::PairBinary),

            c => Err(ErrorKind::UnknownOpcode(c).into()),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum HandshakeStatus {
    Ok = 0x01,
    UserExists = 0x02,
}

impl HandshakeStatus {
    pub fn as_u8(&self) -> u8 {
        *self as u8
    }

    pub fn from_u8(c: u8) -> Result<HandshakeStatus> {
        match c {
            0x01 => Ok(HandshakeStatus::Ok),
            0x02 => Ok(HandshakeStatus::UserExists),

            c => Err(ErrorKind::UnknownHandshakeStatus(c).into()),
        }
    }
}

impl fmt::Display for HandshakeStatus {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            HandshakeStatus::Ok => fmt.write_str("ok"),
            HandshakeStatus::UserExists => fmt.write_str("user-exists"),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ConnectionStatus {
    Ok = 0x01,
    AlreadyConnected = 0x02,
    PairEmployed = 0x03,
    NoSuchUser = 0x04,
    IncorrectPassword = 0x05,
}

impl ConnectionStatus {
    pub fn as_u8(&self) -> u8 {
        *self as u8
    }

    pub fn from_u8(c: u8) -> Result<ConnectionStatus> {
        match c {
            0x01 => Ok(ConnectionStatus::Ok),
            0x02 => Ok(ConnectionStatus::AlreadyConnected),
            0x03 => Ok(ConnectionStatus::PairEmployed),
            0x04 => Ok(ConnectionStatus::NoSuchUser),
            0x05 => Ok(ConnectionStatus::IncorrectPassword),

            c => Err(ErrorKind::UnknownConnectionStatus(c).into()),
        }
    }
}

impl fmt::Display for ConnectionStatus {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ConnectionStatus::Ok => fmt.write_str("ok"),
            ConnectionStatus::AlreadyConnected => fmt.write_str("already-connected"),
            ConnectionStatus::PairEmployed => fmt.write_str("pair-employed"),
            ConnectionStatus::NoSuchUser => fmt.write_str("no-such-user"),
            ConnectionStatus::IncorrectPassword => fmt.write_str("incorrect-password"),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum DisconnectionStatus {
    Ok = 0x01,
    NotConnected = 0x02,
}

impl DisconnectionStatus {
    pub fn as_u8(&self) -> u8 {
        *self as u8
    }

    pub fn from_u8(c: u8) -> Result<DisconnectionStatus> {
        match c {
            0x01 => Ok(DisconnectionStatus::Ok),
            0x02 => Ok(DisconnectionStatus::NotConnected),

            c => Err(ErrorKind::UnknownConnectionStatus(c).into()),
        }
    }
}

impl fmt::Display for DisconnectionStatus {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            DisconnectionStatus::Ok => fmt.write_str("ok"),
            DisconnectionStatus::NotConnected => fmt.write_str("not-connected"),
        }
    }
}
