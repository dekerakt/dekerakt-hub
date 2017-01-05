use std::io::Write;
use byteorder::{BigEndian, WriteBytesExt};

use message::{Message, AuthResult};

pub trait EncodeExt: Write {  // No error handling
    fn encode_u8(&mut self, n: u8) {
        self.write_u8(n).unwrap()
    }

    fn encode_u16(&mut self, n: u16) {
        self.write_u16::<BigEndian>(n).unwrap()
    }

    fn encode_u24(&mut self, n: u64) {
        self.write_uint::<BigEndian>(n, 3).unwrap()
    }

    fn encode_u32(&mut self, n: u32) {
        self.write_u32::<BigEndian>(n).unwrap()
    }

    fn encode_u64(&mut self, n: u64) {
        self.write_u64::<BigEndian>(n).unwrap()
    }

    fn encode_auth_result(&mut self, result: AuthResult) {
        self.encode_u8(result as u8)
    }

    fn encode_string(&mut self, string: String) {
        self.encode_u24(string.len() as u64);
        self.write_all(string.as_ref()).unwrap();
    }

    fn encode_message(&mut self, message: Message) {
        let ref mut body = Vec::with_capacity(8);
        let code;

        match message {
            Message::Error {
                description
            } => {
                code = 0x00;
                body.encode_string(description);
            }

            Message::AuthServer {
                auth_result,
                display_message
            } => {
                code = 0x02;
                body.encode_auth_result(auth_result);
                body.encode_string(display_message);
            }

            _ => unimplemented!()
        }

        self.encode_u8(code);
        self.encode_u24(body.len() as u64);
        self.write_all(body.as_slice()).unwrap();
    }
}

impl<T: Write> EncodeExt for T {}
