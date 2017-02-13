use graphics::{Palette, ScreenResolution, ScreenState, PreciseMode,
    Char, Color};

use std::time::Duration;

#[derive(Debug, Clone)]
pub enum Message {
    Error {
        description: String
    },
    AuthClient {
        user: String,
        password: String,
        connection_mode: ConnectionMode,
        connection_side: ConnectionSide,
        ping_interval: Duration
    },
    AuthServer {
        auth_result: AuthResult,
        display_message: String
    },
    InitialData {
        palette: Palette,
        fg_idx: u8,
        bg_idx: u8,
        resolution: ScreenResolution,
        screen_state: ScreenState,
        precise_mode: PreciseMode,
        chars: Vec<Char>
    },
    SetBG {
        index: u8
    },
    SetFG {
        index: u8
    },
    SetPalette {
        color: Color,
        index: u8
    },
    SetResolution {
        resolution: ScreenResolution
    },
    SetChars {
        x: u8,
        y: u8,
        chars: String,
        vertical: bool
    },
    Copy {
        x: u8,
        y: u8,
        w: u8,
        h: u8,
        tx: u8,
        ty: u8
    },
    Fill {
        x: u8,
        y: u8,
        w: u8,
        h: u8,
        char: char
    },
    SetScreenState {
        screen_state: ScreenState
    },
    SetPrecise {
        precise_mode: PreciseMode
    },
    Fetch,
    EventTouch {
        x: u8,
        y: u8,
        button: i8
    },
    EventDrag {
        x: u8,
        y: u8,
        button: i8
    },
    EventDrop {
        x: u8,
        y: u8,
        button: i8
    },
    EventScroll {
        x: u8,
        y: u8,
        direction: Direction,
        delta: u8
    },
    EventKeyDown {
        char_code: u32,
        lwjgl_code: u32
    },
    EventKeyUp {
        char_code: u32,
        lwjgl_code: u32
    },
    EventClipboard {
        data: String
    },
    Ping {
        ping: u64
    },
    Pong {
        pong: u64
    }
}

#[derive(Debug, Copy, Clone)]
pub enum ConnectionMode {
    GpuKbd = 0x00,
    Gpu = 0x01,
    Kbd = 0x02,
    Custom = 0x03
}

#[derive(Debug, Copy, Clone)]
pub enum ConnectionSide {
    OC = 0x00,
    External = 0x01,
    Custom = 0xfe,
    None = 0xff
}

#[derive(Debug, Copy, Clone)]
pub enum AuthResult {
    Authenticated = 0x00,
    BadCredentials = 0x01,
    UnsupportedMode = 0x02,
    VertexInUse = 0x03
}

#[derive(Debug, Copy, Clone)]
pub enum Direction {
    Up = 0xff,
    Down = 0x00
}
