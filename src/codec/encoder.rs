use std::io::Write;
use byteorder::{BigEndian, WriteBytesExt};

use message::{Message, AuthResult};

fn encode_u8(buf: &mut Vec<u8>, n: u8) {
    buf.write_u8(n).unwrap()
}

fn encode_u16(buf: &mut Vec<u8>, n: u16) {
    buf.write_u16::<BigEndian>(n).unwrap()
}

fn encode_u24(buf: &mut Vec<u8>, n: u64) {
    buf.write_uint::<BigEndian>(n, 3).unwrap()
}

fn encode_u32(buf: &mut Vec<u8>, n: u32) {
    buf.write_u32::<BigEndian>(n).unwrap()
}

fn encode_u64(buf: &mut Vec<u8>, n: u64) {
    buf.write_u64::<BigEndian>(n).unwrap()
}

fn encode_auth_result(buf: &mut Vec<u8>, result: AuthResult) {
    encode_u8(buf, result as u8)
}

fn encode_string(buf: &mut Vec<u8>, string: String) {
    encode_u24(buf, string.len() as u64);
    buf.write_all(string.as_ref()).unwrap();
}

fn encode(buf: &mut Vec<u8>, message: Message) {
    let ref mut body = Vec::with_capacity(8);
    let code;

    match message {
        Message::AuthServer {
            auth_result,
            display_message
        } => {
            code = 0x02;
            encode_auth_result(body, auth_result);
            encode_string(body, display_message);
        }

        _ => unimplemented!()
    }

    encode_u8(buf, code);
    encode_u24(buf, body.len() as u64);
    buf.write_all(body.as_slice()).unwrap();
}

