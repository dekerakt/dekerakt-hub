use std::io;
use std::io::Read;
use std::time::Duration;

use byteorder::{BigEndian, ReadBytesExt};
use tokio_core::io::EasyBuf;

use super::IoOption;
use message::{Message, ConnectionMode};

fn decode_u8(mut buf: &[u8]) -> IoOption<u8> {
    Ok(buf.read_u8().ok())
}

fn decode_u16(mut buf: &[u8]) -> IoOption<u16> {
    Ok(buf.read_u16::<BigEndian>().ok())
}

fn decode_u24(mut buf: &[u8]) -> IoOption<u64> {
    Ok(buf.read_uint::<BigEndian>(3).ok())
}

fn decode_u32(mut buf: &[u8]) -> IoOption<u32> {
    Ok(buf.read_u32::<BigEndian>().ok())
}

fn decode_u64(mut buf: &[u8]) -> IoOption<u64> {
    Ok(buf.read_u64::<BigEndian>().ok())
}

fn decode_connection_mode(mut buf: &[u8]) -> IoOption<ConnectionMode> {
    match try_io_opt!(decode_u8(buf)) {
        0x00 => Ok(Some(ConnectionMode::GpuKbd)),
        0x01 => Ok(Some(ConnectionMode::Gpu)),
        0x02 => Ok(Some(ConnectionMode::Kbd)),
        0x03 => Ok(Some(ConnectionMode::Custom)),

        _ => Err(io::Error::new(io::ErrorKind::Other, "invalid connection mode"))
    }
}

fn decode_duration(mut buf: &[u8]) -> IoOption<Duration> {
    Ok(Some(Duration::from_secs(try_io_opt!(decode_u16(buf)) as u64)))
}

fn decode_string(mut buf: &[u8]) -> IoOption<String> {
    let len = try_io_opt!(decode_u24(buf)) as usize;

    let mut string_buf = Vec::with_capacity(len);
    buf.read_exact(string_buf.as_mut_slice());

    match String::from_utf8(string_buf) {
        Ok(v) => Ok(Some(v)),
        Err(_) => Err(io::Error::new(io::ErrorKind::Other, "invalid string"))
    }
}

pub fn decode(buf: &mut EasyBuf) -> IoOption<Message> {
    let mut buf = buf.as_ref();
    let code = try_io_opt!(decode_u8(buf));
    let len = try_io_opt!(decode_u24(buf)) as usize;

    let mut body_buf = Vec::with_capacity(len);
    buf.read_exact(body_buf.as_mut_slice());
    let mut body_buf = body_buf.as_slice();

    match code {
        0x01 => Ok(Some(Message::AuthClient {
            user: try_io_opt!(decode_string(body_buf)),
            password: try_io_opt!(decode_string(body_buf)),
            connection_mode: try_io_opt!(decode_connection_mode(body_buf)),
            ping_interval: try_io_opt!(decode_duration(body_buf))
        })),

        _ => unimplemented!()
    }
}

