pub mod decoder;
pub mod encoder;

use std::{io, fmt, str};
use std::str::Utf8Error;

use tokio_core::io::{Codec as CodecTrait, EasyBuf};

use message::Message;
use self::decoder::{DecodeExt, DecodeError, DecodeResult};
use self::encoder::EncodeExt;

pub type IoOption<T> = io::Result<Option<T>>;

pub struct Codec;

impl CodecTrait for Codec {
    type In = Result<Message, DecodeError>;
    type Out = Message;

    fn decode(&mut self, buf: &mut EasyBuf) -> io::Result<Option<Self::In>> {
        let (result, len) = {
            let mut buf_slice = buf.as_slice();
            let was = buf_slice.len();

            let result = match buf_slice.decode_message() {
                DecodeResult::Ok(v) => Ok(Some(Ok(v))),
                DecodeResult::Err(e) => Ok(Some(Err(e))),

                DecodeResult::IoErr(ref e)
                    if e.kind() == io::ErrorKind::UnexpectedEof => return Ok(None),
                DecodeResult::IoErr(e) => {
                    crit!("got DecodeResult::IoErr({:?}) wut", e);
                    return Err(e);
                }
            };

            (result, was - buf_slice.len())
        };

        buf.drain_to(len);
        result
    }

    fn encode(&mut self, msg: Message, buf: &mut Vec<u8>) -> io::Result<()> {
        buf.encode_message(msg);

        Ok(())
    }
}

