use std::io::Write;
use byteorder::{BigEndian, WriteBytesExt};

use message::{Message, AuthResult};
use graphics::{Color, Palette, ScreenResolution, ScreenState,
    PreciseMode, Char};

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

    fn encode_screen_state(&mut self, screen_state: ScreenState) {
        self.encode_u8(screen_state as u8)
    }

    fn encode_precise_mode(&mut self, precise_mode: PreciseMode) {
        self.encode_u8(precise_mode as u8)
    }

    fn encode_screen_resolution(&mut self, screen_resolution: ScreenResolution) {
        self.encode_u8(screen_resolution.width);
        self.encode_u8(screen_resolution.height)
    }

    fn encode_color(&mut self, color: Color) {
        self.encode_u8(color.r);
        self.encode_u8(color.g);
        self.encode_u8(color.b)
    }

    fn encode_palette(&mut self, palette: Palette) {
        for &color in palette.iter() {
            self.encode_color(color);
        }
    }

    fn encode_char_raw(&mut self, char_raw: char) {
        self.encode_u32(char_raw as u32)
    }

    fn encode_char(&mut self, char: Char) {
        self.encode_u8(char.fg_idx);
        self.encode_u8(char.bg_idx);
        self.encode_char_raw(char.char);
    }

    fn encode_chars(&mut self, chars: Vec<Char>) {
        for char in chars {
            self.encode_char(char);
        }
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

            Message::InitialData {
                palette,
                fg_idx,
                bg_idx,
                resolution,
                screen_state,
                precise_mode,
                chars
            } => {
                code = 0x03;

                body.encode_palette(palette);
                body.encode_u8(fg_idx);
                body.encode_u8(bg_idx);
                body.encode_screen_resolution(resolution);
                body.encode_screen_state(screen_state);
                body.encode_precise_mode(precise_mode);
                body.encode_chars(chars);
            }

            _ => unimplemented!()
        }

        self.encode_u8(code);
        self.encode_u24(body.len() as u64);
        self.write_all(body.as_slice()).unwrap();
    }
}

impl<T: Write> EncodeExt for T {}
