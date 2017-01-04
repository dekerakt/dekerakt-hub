use std::{io, fmt, str};
use std::io::Result as IoResult;
use std::str::Utf8Error;

use tokio_core::io::{Codec as CodecTrait, EasyBuf};

#[macro_use]
mod macros;
mod decoder;
mod encoder;

pub type IoOption<T> = IoResult<Option<T>>;

pub struct Codec;

impl CodecTrait for Codec {
    type In = Result<String, Error>;
    type Out = String;

    fn decode(&mut self, buf: &mut EasyBuf) -> io::Result<Option<Self::In>> {
        if let Some(pos) = buf.as_ref().iter().position(|&b| b == b'\n') {
            let line = buf.drain_to(pos);
            buf.drain_to(1);


            return match str::from_utf8(line.as_ref()) {
                Ok(v) => {
                    Ok(Some(Ok(v.to_string())))
                }

                Err(e) => {
                    Ok(Some(Err(Error::InvalidString(e))))
                }
            };
        }

        Ok(None)
    }

    fn encode(&mut self, msg: String, buf: &mut Vec<u8>) -> io::Result<()> {

        for &byte in msg.as_bytes() {
            buf.push(byte);
        }

        buf.push(b'\n');

        Ok(())
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Error {
    InvalidString(Utf8Error)
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::InvalidString(e) => write!(fmt, "invalid string ({})", e)
        }
    }
}

