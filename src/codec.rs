use std::str;
use std::io::Cursor;

use bytes::{Buf, BufMut, BytesMut, BigEndian};
use error::{Error, ErrorKind, Result};
use protocol::{Message, Opcode, HandshakeStatus, ConnectionStatus, DisconnectionStatus};

macro_rules! try_decode {
    ($e:expr) => (match $e {
        Ok(Some(v)) => v,
        Ok(None) => return Ok(None),
        Err(e) => return Err(e)
    })
}

type ByteOrder = BigEndian;

pub struct Codec;

impl Codec {
    fn decode_u8<B: Buf>(&mut self, buf: &mut B) -> Result<Option<u8>> {
        if buf.remaining() < 1 {
            Ok(None)
        } else {
            Ok(Some(buf.get_u8()))
        }
    }

    fn decode_u32<B: Buf>(&mut self, buf: &mut B) -> Result<Option<u32>> {
        if buf.remaining() < 4 {
            Ok(None)
        } else {
            Ok(Some(buf.get_u32::<ByteOrder>()))
        }
    }

    fn decode_u64<B: Buf>(&mut self, buf: &mut B) -> Result<Option<u64>> {
        if buf.remaining() < 8 {
            Ok(None)
        } else {
            Ok(Some(buf.get_u64::<ByteOrder>()))
        }
    }

    fn decode_bytes<B: Buf>(&mut self, buf: &mut B) -> Result<Option<Vec<u8>>> {
        let len = try_decode!(self.decode_u32(buf)) as usize;

        if buf.remaining() < len {
            Ok(None)
        } else {
            let mut v = Vec::with_capacity(len);
            v.extend_from_slice(&buf.bytes()[..len]);
            buf.advance(len);
            Ok(Some(v))
        }
    }

    fn decode_string<B: Buf>(&mut self, buf: &mut B) -> Result<Option<String>> {
        let bytes = try_decode!(self.decode_bytes(buf));

        match String::from_utf8(bytes) {
            Ok(v) => Ok(Some(v)),
            Err(e) => Err(e.utf8_error().into()),
        }
    }

    fn decode_opcode<B: Buf>(&mut self, buf: &mut B) -> Result<Option<Opcode>> {
        let b = try_decode!(self.decode_u8(buf));
        Ok(Some(Opcode::from_u8(b)?))
    }

    fn decode_handshake_status<B: Buf>(&mut self, buf: &mut B) -> Result<Option<HandshakeStatus>> {
        let b = try_decode!(self.decode_u8(buf));
        Ok(Some(HandshakeStatus::from_u8(b)?))
    }

    fn decode_connection_status<B: Buf>(&mut self,
                                        buf: &mut B)
                                        -> Result<Option<ConnectionStatus>> {
        let b = try_decode!(self.decode_u8(buf));
        Ok(Some(ConnectionStatus::from_u8(b)?))
    }

    fn decode_disconnection_status<B: Buf>(&mut self,
                                           buf: &mut B)
                                           -> Result<Option<DisconnectionStatus>> {
        let b = try_decode!(self.decode_u8(buf));
        Ok(Some(DisconnectionStatus::from_u8(b)?))
    }

    fn decode_message<B: Buf>(&mut self, buf: &mut B) -> Result<Option<Message>> {
        let opcode = try_decode!(self.decode_opcode(buf));

        match opcode {
            Opcode::Error => Ok(Some(Message::Error(try_decode!(self.decode_string(buf))))),

            Opcode::CriticalError => {
                Ok(Some(Message::CriticalError(try_decode!(self.decode_string(buf)))))
            }

            Opcode::Ping => Ok(Some(Message::Ping(try_decode!(self.decode_u64(buf))))),
            Opcode::Pong => Ok(Some(Message::Pong(try_decode!(self.decode_u64(buf))))),

            Opcode::ClientHandshake => {
                Ok(Some(Message::ClientHandshake {
                            username: try_decode!(self.decode_string(buf)),
                            password: try_decode!(self.decode_string(buf)),
                        }))
            }

            Opcode::ClientConnect => {
                Ok(Some(Message::ClientConnect {
                            username: try_decode!(self.decode_string(buf)),
                            password: try_decode!(self.decode_string(buf)),
                        }))
            }

            Opcode::ClientDisconnect => Ok(Some(Message::ClientDisconnect)),
            Opcode::ClientGoodbye => Ok(Some(Message::ClientGoodbye)),

            Opcode::ServerHandshake => {
                Ok(Some(Message::ServerHandshake(try_decode!(self.decode_handshake_status(buf)))))
            }

            Opcode::ServerConnect => {
                Ok(Some(Message::ServerConnect(try_decode!(self.decode_connection_status(buf)))))
            }

            Opcode::ServerDisconnect => {
                Ok(Some(Message::ServerDisconnect(
                            try_decode!(self.decode_disconnection_status(buf)))))
            }

            Opcode::ServerGoodbye => Ok(Some(Message::ServerGoodbye)),

            Opcode::PairPing => Ok(Some(Message::PairPing(try_decode!(self.decode_u64(buf))))),
            Opcode::PairPong => Ok(Some(Message::PairPong(try_decode!(self.decode_u64(buf))))),
            Opcode::PairText => Ok(Some(Message::PairText(try_decode!(self.decode_string(buf))))),

            Opcode::PairBinary => {
                Ok(Some(Message::PairBinary(try_decode!(self.decode_bytes(buf)))))
            }
        }
    }
}

pub fn decode(buf: &mut BytesMut) -> Result<Option<Message>> {
    let mut codec = Codec;

    let (result, amt) = {
        let mut buf = Cursor::new(&buf);
        let result = codec.decode_message(&mut buf);

        (result, buf.position())
    };

    match result {
        Ok(Some(..)) | Err(..) => drop(buf.split_to(amt as usize)),
        _ => (),
    }

    result
}

impl Codec {
    fn encode_u8<B: BufMut>(&mut self, n: u8, buf: &mut B) -> Result<()> {
        if buf.remaining_mut() < 1 {
            return Err(ErrorKind::BufferOverflow.into());
        }

        buf.put_u8(n);

        Ok(())
    }

    fn encode_u64<B: BufMut>(&mut self, n: u64, buf: &mut B) -> Result<()> {
        if buf.remaining_mut() < 8 {
            return Err(ErrorKind::BufferOverflow.into());
        }

        buf.put_u64::<ByteOrder>(n);

        Ok(())
    }

    fn encode_bytes<B: BufMut>(&mut self, bytes: &[u8], buf: &mut B) -> Result<()> {
        if buf.remaining_mut() < bytes.len() + 4 {
            return Err(ErrorKind::BufferOverflow.into());
        }

        buf.put_u32::<ByteOrder>(bytes.len() as u32);
        buf.put_slice(bytes);

        Ok(())
    }

    fn encode_string<B: BufMut>(&mut self, string: &str, buf: &mut B) -> Result<()> {
        self.encode_bytes(string.as_bytes(), buf)
    }

    fn encode_opcode<B: BufMut>(&mut self, c: Opcode, buf: &mut B) -> Result<()> {
        self.encode_u8(c.as_u8(), buf)
    }

    fn encode_handshake_status<B: BufMut>(&mut self,
                                          c: HandshakeStatus,
                                          buf: &mut B)
                                          -> Result<()> {
        self.encode_u8(c.as_u8(), buf)
    }

    fn encode_connection_status<B: BufMut>(&mut self,
                                           c: ConnectionStatus,
                                           buf: &mut B)
                                           -> Result<()> {
        self.encode_u8(c.as_u8(), buf)
    }

    fn encode_disconnection_status<B: BufMut>(&mut self,
                                              c: DisconnectionStatus,
                                              buf: &mut B)
                                              -> Result<()> {
        self.encode_u8(c.as_u8(), buf)
    }

    fn encode_message<B: BufMut>(&mut self, message: Message, buf: &mut B) -> Result<()> {
        self.encode_opcode(message.opcode(), buf)?;

        match message {
            Message::Error(s) => self.encode_string(&s, buf),
            Message::CriticalError(s) => self.encode_string(&s, buf),
            Message::Ping(t) => self.encode_u64(t, buf),
            Message::Pong(t) => self.encode_u64(t, buf),

            Message::ClientHandshake { username, password } => {
                self.encode_string(&username, buf)?;
                self.encode_string(&password, buf)
            }

            Message::ClientConnect { username, password } => {
                self.encode_string(&username, buf)?;
                self.encode_string(&password, buf)
            }

            Message::ClientDisconnect => Ok(()),
            Message::ClientGoodbye => Ok(()),

            Message::ServerHandshake(s) => self.encode_handshake_status(s, buf),
            Message::ServerConnect(s) => self.encode_connection_status(s, buf),
            Message::ServerDisconnect(s) => self.encode_disconnection_status(s, buf),
            Message::ServerGoodbye => Ok(()),

            Message::PairPing(t) => self.encode_u64(t, buf),
            Message::PairPong(t) => self.encode_u64(t, buf),
            Message::PairText(s) => self.encode_string(&s, buf),
            Message::PairBinary(b) => self.encode_bytes(&b, buf),
        }
    }
}

pub fn encode(msg: Message, buf: &mut BytesMut) -> Result<()> {
    let mut codec = Codec;
    codec.encode_message(msg, buf)
}

