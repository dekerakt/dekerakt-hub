use std::{io, fmt};
use std::io::Read;
use std::time::Duration;
use std::str::Utf8Error;

use byteorder::{BigEndian, ReadBytesExt};
use tokio_core::io::EasyBuf;

use message::{Message, ConnectionMode, ConnectionSide};

macro_rules! try_decode {
    ($e:expr) => (match $e {
        DecodeResult::Ok(v) => v,
        DecodeResult::Err(e) => return DecodeResult::Err(e),
        DecodeResult::IoErr(e) => return DecodeResult::IoErr(e)
    })
}

macro_rules! try_io_decode {
    ($e:expr) => (match $e {
        Ok(v) => v,
        Err(e) => return DecodeResult::IoErr(e)
    })
}

#[derive(Debug, Copy, Clone)]
pub enum DecodeError {
    InvalidString(Utf8Error),
    InvalidConnectionMode,
    InvalidConnectionSide,
    UnknownMessage
}

impl fmt::Display for DecodeError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            DecodeError::InvalidString(e) =>
                write!(fmt, "invalid string ({})", e),
            DecodeError::InvalidConnectionMode =>
                write!(fmt, "invalid connection mode"),
            DecodeError::InvalidConnectionSide =>
                write!(fmt, "invalid connection side"),
            DecodeError::UnknownMessage =>
                write!(fmt, "unknown message")
        }
    }
}

pub enum DecodeResult<T> {
    Ok(T),
    Err(DecodeError),
    IoErr(io::Error)
}

impl<T> DecodeResult<T> {
    fn map<U, F>(self, f: F) -> DecodeResult<U>
        where F: Fn(T) -> U
    {
        DecodeResult::Ok(f(try_decode!(self)))
    }
}

impl<T> From<T> for DecodeResult<T> {
    fn from(inner: T) -> DecodeResult<T> {
        DecodeResult::Ok(inner)
    }
}

impl<T> From<io::Result<T>> for DecodeResult<T> {
    fn from(inner: io::Result<T>) -> DecodeResult<T> {
        match inner {
            Ok(v) => DecodeResult::Ok(v),
            Err(e) => DecodeResult::IoErr(e)
        }
    }
}

pub trait DecodeExt: Read {
    fn decode_u8(&mut self) -> DecodeResult<u8> {
        self.read_u8().map(|v| {
            debug!("decoded {} u8", v);
            v
        }).into()
    }

    fn decode_u16(&mut self) -> DecodeResult<u16> {
        self.read_u16::<BigEndian>().map(|v| {
            debug!("decoded {} u16", v);
            v
        }).into()
    }

    fn decode_u24(&mut self) -> DecodeResult<u64> {
        self.read_uint::<BigEndian>(3).map(|v| {
            debug!("decoded {} u24", v);
            v
        }).into()
    }

    fn decode_u32(&mut self) -> DecodeResult<u32> {
        self.read_u32::<BigEndian>().map(|v| {
            debug!("decoded {} u32", v);
            v
        }).into()
    }

    fn decode_u64(&mut self) -> DecodeResult<u64> {
        self.read_u64::<BigEndian>().map(|v| {
            debug!("decoded {} u64", v);
            v
        }).into()
    }

    fn decode_connection_mode(&mut self) -> DecodeResult<ConnectionMode> {
        match try_decode!(self.decode_u8()) {
            0x00 => ConnectionMode::GpuKbd.into(),
            0x01 => ConnectionMode::Gpu.into(),
            0x02 => ConnectionMode::Kbd.into(),
            0x03 => ConnectionMode::Custom.into(),

            _ => DecodeResult::Err(DecodeError::InvalidConnectionMode)
        }.map(|v| {
            debug!("decoded {:?}", v);
            v
        })
    }

    fn decode_connection_side(&mut self) -> DecodeResult<ConnectionSide> {
        match try_decode!(self.decode_u8()) {
            0x00 => ConnectionSide::OC.into(),
            0x01 => ConnectionSide::External.into(),
            0xff => ConnectionSide::Custom.into(),

            _ => DecodeResult::Err(DecodeError::InvalidConnectionSide)
        }.map(|v| {
            debug!("decoded {:?}", v);
            v
        })
    }

    fn decode_duration(&mut self) -> DecodeResult<Duration> {
        let v: DecodeResult<_> = Duration::from_secs(
            try_decode!(self.decode_u16()) as u64).into();

        v.map(|v| {
            debug!("decoded {:?}", v);
            v
        })
    }

    fn decode_string(&mut self) -> DecodeResult<String> {
        let len = try_decode!(self.decode_u24()) as usize;

        let mut string_buf = Vec::with_capacity(len);
        try_io_decode!(self.read_exact(string_buf.as_mut_slice()));

        match String::from_utf8(string_buf) {
            Ok(v) => v.into(),
            Err(e) =>
                DecodeResult::Err(DecodeError::InvalidString(e.utf8_error()))
        }.map(|v| {
            debug!("decoded {:?}", v);
            v
        })
    }

    fn decode_message(&mut self) -> DecodeResult<Message> {
        let code = try_decode!(self.decode_u8());
        let len = try_decode!(self.decode_u24()) as usize;

        let mut body_buf = vec![0; len];
        try_io_decode!(self.read_exact(body_buf.as_mut_slice()));
        let mut body_buf = body_buf.as_slice();

        debug!("body {}", body_buf.len());

        match code {
            0x01 => Message::AuthClient {
                user: try_decode!(body_buf.decode_string()),
                password: try_decode!(body_buf.decode_string()),
                connection_mode: try_decode!(body_buf.decode_connection_mode()),
                connection_side: try_decode!(body_buf.decode_connection_side()),
                ping_interval: try_decode!(body_buf.decode_duration())
            }.into(),

            _ => DecodeResult::Err(DecodeError::UnknownMessage)
        }
    }
}

impl<T: Read> DecodeExt for T {}
