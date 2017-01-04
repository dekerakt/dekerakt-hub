use std::{io, str};
use std::io::Result as IoResult;

use tokio_core::io::{Codec as CodecTrait, EasyBuf};

#[macro_use]
mod macros;
mod decoder;
mod encoder;

pub type IoOption<T> = IoResult<Option<T>>;

pub struct Codec;

impl CodecTrait for Codec {
    type In = String;
    type Out = String;

    fn decode(&mut self, buf: &mut EasyBuf) -> io::Result<Option<String>> {
        if let Some(pos) = buf.as_ref().iter().position(|&b| b == b'\n') {
            let line = buf.drain_to(pos);
            buf.drain_to(1);


            return match str::from_utf8(line.as_ref()) {
                Ok(v) => {
                    Ok(Some(v.to_string()))
                }

                Err(e) => {
                    Err(io::Error::new(io::ErrorKind::Other, "invalid string"))
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

