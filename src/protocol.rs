use std::fmt;
use std::io::Cursor;

use bytes::{self, BytesMut, Buf, BufMut};
use error::{Result, Error, ErrorKind};

type ByteOrder = bytes::BigEndian;

pub fn encode_to_bytesmut<E: Encode>(buf: &mut BytesMut, e: E) {
    e.encode(buf);
}

pub fn decode_from_bytesmut<D: Decode>(buf: &mut BytesMut) -> Result<Option<D>> {
    let (result, len) = {
        let mut buf = Cursor::new(&buf);
        (D::decode(&mut buf), buf.position())
    };

    buf.split_to(len as usize);
    result
}

#[derive(Debug)]
pub enum Message {
    Error(Error),

    ClientUsername(String),
    ClientPassword(String),

    ServerAuth(AuthStatus),
}

impl Message {
    fn opcode(&self) -> Opcode {
        match *self {
            Message::Error(_) => Opcode::Error,

            Message::ClientUsername(_) => Opcode::ClientUsername,
            Message::ClientPassword(_) => Opcode::ClientPassword,

            Message::ServerAuth(_) => Opcode::ServerAuth,
        }
    }
}

impl fmt::Display for Message {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Message::Error(ref e) => write!(fmt, "error[{}]", e),

            Message::ClientUsername(ref s) => write!(fmt, "client-username[{}]", s),
            Message::ClientPassword(_) => write!(fmt, "client-password"),

            Message::ServerAuth(ref status) => write!(fmt, "server-auth[{}]", status)
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum AuthStatus {
    Ok = 0x00,
    UsernameUsed = 0x01,
    IncorrectPassword = 0x02,
}

impl fmt::Display for AuthStatus {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            AuthStatus::Ok => fmt.write_str("ok"),
            AuthStatus::UsernameUsed => fmt.write_str("username-used"),
            AuthStatus::IncorrectPassword => fmt.write_str("incorrect-password")
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Opcode {
    Error = 0x00,

    ClientUsername = 0x01,
    ClientPassword = 0x02,

    ServerAuth = 0x7f,
}

pub trait Encode {
    fn encode<B: BufMut>(&self, buf: &mut B);
}

pub trait Decode: Sized {
    fn decode<B: Buf>(buf: &mut B) -> Result<Option<Self>>;
}

impl Encode for String {
    fn encode<B: BufMut>(&self, buf: &mut B) {
        buf.put_u32::<ByteOrder>(self.len() as u32);
        buf.put(self);
    }
}

impl Decode for String {
    fn decode<B: Buf>(buf: &mut B) -> Result<Option<Self>> {
        if buf.remaining() < 4 {
            return Ok(None);
        }

        let len = buf.get_u32::<ByteOrder>() as usize;

        if buf.remaining() < len {
            return Ok(None);
        }

        let mut v = vec![0; len];
        buf.copy_to_slice(&mut v);
        v.truncate(len);

        match String::from_utf8(v) {
            Ok(v) => Ok(Some(v)),
            Err(e) => Err(e.into()),
        }
    }
}


impl Encode for Message {
    fn encode<B: BufMut>(&self, buf: &mut B) {
        self.opcode().encode(buf);

        match *self {
            Message::Error(ref e) => format!("{}", e).encode(buf),

            Message::ClientUsername(ref s) => s.encode(buf),
            Message::ClientPassword(ref s) => s.encode(buf),

            Message::ServerAuth(ref a) => a.encode(buf),
        }
    }
}

impl Decode for Message {
    fn decode<B: Buf>(buf: &mut B) -> Result<Option<Self>> {
        let opcode = match Opcode::decode(buf) {
            Ok(Some(v)) => v,
            Ok(None) => return Ok(None),
            Err(e) => return Err(e),
        };

        match opcode {
            Opcode::Error => {
                match String::decode(buf) {
                    Ok(Some(v)) => Ok(Some(Message::Error(v.into()))),
                    Ok(None) => Ok(None),
                    Err(e) => Err(e),
                }
            }

            Opcode::ClientUsername => {
                match String::decode(buf) {
                    Ok(Some(v)) => Ok(Some(Message::ClientUsername(v))),
                    Ok(None) => Ok(None),
                    Err(e) => Err(e),
                }
            }

            Opcode::ClientPassword => {
                match String::decode(buf) {
                    Ok(Some(v)) => Ok(Some(Message::ClientUsername(v))),
                    Ok(None) => Ok(None),
                    Err(e) => Err(e),
                }
            }

            Opcode::ServerAuth => {
                match String::decode(buf) {
                    Ok(Some(v)) => Ok(Some(Message::ClientUsername(v))),
                    Ok(None) => Ok(None),
                    Err(e) => Err(e),
                }
            }
        }
    }
}


impl Encode for AuthStatus {
    fn encode<B: BufMut>(&self, buf: &mut B) {
        buf.put_u8(*self as u8)
    }
}

impl Decode for AuthStatus {
    fn decode<B: Buf>(buf: &mut B) -> Result<Option<AuthStatus>> {
        if !buf.has_remaining() {
            return Ok(None);
        }

        match buf.get_u8() {
            0x00 => Ok(Some(AuthStatus::Ok)),
            0x01 => Ok(Some(AuthStatus::UsernameUsed)),
            0x02 => Ok(Some(AuthStatus::IncorrectPassword)),
            b => Err(ErrorKind::UnknownAuthStatus(b).into()),
        }
    }
}

impl Encode for Opcode {
    fn encode<B: BufMut>(&self, buf: &mut B) {
        buf.put_u8(*self as u8)
    }
}

impl Decode for Opcode {
    fn decode<B: Buf>(buf: &mut B) -> Result<Option<Opcode>> {
        if !buf.has_remaining() {
            return Ok(None);
        }

        match buf.get_u8() {
            0x00 => Ok(Some(Opcode::Error)),

            0x01 => Ok(Some(Opcode::ClientUsername)),
            0x02 => Ok(Some(Opcode::ClientPassword)),

            0x7f => Ok(Some(Opcode::ServerAuth)),

            b => Err(ErrorKind::UnknownOpcode(b).into()),
        }
    }
}

