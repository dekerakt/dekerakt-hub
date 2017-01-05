use std::io::Write;
use byteorder::{BigEndian, WriteBytesExt};

use message::{Message, AuthResult, Direction};
use graphics::{Color, Palette, ScreenResolution, ScreenState,
    PreciseMode, Char};

pub trait EncodeExt: Write {  // No error handling
    fn encode_u8(&mut self, n: u8) {
        self.write_u8(n).unwrap()
    }

    fn encode_i8(&mut self, n: i8) {
        self.write_i8(n).unwrap()
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

    fn encode_direction(&mut self, direction: Direction) {
        self.encode_u8(direction as u8)
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

    fn encode_bool(&mut self, value: bool) {
        self.encode_u8(match value {
            true => 0xff,
            false => 0x00
        })
    }

    fn encode_message(&mut self, message: Message) {
        let ref mut body = Vec::with_capacity(8);
        let code;

        match message {
            Message::Error { description } => {
                code = 0x00;
                body.encode_string(description);
            }

            Message::AuthServer { auth_result, display_message } => {
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

            Message::SetBG { index } => {
                code = 0x04;
                body.encode_u8(index);
            }

            Message::SetFG { index } => {
                code = 0x05;
                body.encode_u8(index);
            }

            Message::SetPalette { color, index } => {
                code = 0x06;
                body.encode_color(color);
                body.encode_u8(index);
            }

            Message::SetResolution { resolution } => {
                code = 0x07;
                body.encode_screen_resolution(resolution);
            }

            Message::SetChars { x, y, chars, vertical } => {
                code = 0x08;
                body.encode_u8(x);
                body.encode_u8(y);
                body.encode_string(chars);
                body.encode_bool(vertical);
            }

            Message::Copy { x, y, w, h, tx, ty } => {
                code = 0x09;
                body.encode_u8(x);
                body.encode_u8(y);
                body.encode_u8(w);
                body.encode_u8(h);
                body.encode_u8(tx);
                body.encode_u8(ty);
            }

            Message::Fill { x, y, w, h, char } => {
                code = 0x0a;
                body.encode_u8(x);
                body.encode_u8(y);
                body.encode_u8(w);
                body.encode_u8(h);
                body.encode_char_raw(char);
            }

            Message::SetScreenState { screen_state } => {
                code = 0x0b;
                body.encode_screen_state(screen_state);
            }

            Message::SetPrecise { precise_mode } => {
                code = 0x0c;
                body.encode_precise_mode(precise_mode);
            }

            Message::Fetch => {
                code = 0x0d;
            }

            Message::EventTouch { x, y, button } => {
                code = 0x0e;
                body.encode_u8(x);
                body.encode_u8(y);
                body.encode_i8(button);
            }

            Message::EventDrag { x, y, button } => {
                code = 0x0f;
                body.encode_u8(x);
                body.encode_u8(y);
                body.encode_i8(button);
            }

            Message::EventDrop { x, y, button } => {
                code = 0x10;
                body.encode_u8(x);
                body.encode_u8(y);
                body.encode_i8(button);
            }

            Message::EventScroll { x, y, direction, delta } => {
                code = 0x11;
                body.encode_u8(x);
                body.encode_u8(x);
                body.encode_direction(direction);
                body.encode_u8(delta);
            }

            Message::EventKeyDown { char_code, lwjgl_code } => {
                code = 0x12;
                body.encode_u32(char_code);
                body.encode_u32(lwjgl_code);
            }

            Message::EventKeyUp { char_code, lwjgl_code } => {
                code = 0x13;
                body.encode_u32(char_code);
                body.encode_u32(lwjgl_code);
            }

            Message::EventClipboard { data } => {
                code = 0x14;
                body.encode_string(data);
            }

            Message::Ping { ping } => {
                code = 0x15;
                body.encode_u64(ping);
            }

            Message::Pong { pong } => {
                code = 0x16;
                body.encode_u64(pong);
            }

            _ => unimplemented!()
        }

        self.encode_u8(code);
        self.encode_u24(body.len() as u64);
        self.write_all(body.as_slice()).unwrap();
    }
}

impl<T: Write> EncodeExt for T {}
