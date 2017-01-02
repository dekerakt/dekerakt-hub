pub enum Message {
    Error {
        description: String
    },
    AuthClient {
        user: String,
        password: String,
        connectionMode: ConnectionMode,
        pingInterval: std::time::Duration
    },
    AuthServer {
        authResult: AuthResult,
        displayMessage: String
    },
    InitialData {
        palette: Palette,
        fgIdx: u8,
        bgIdx: u8,
        resolution: ScreenResolution,
        screenState: ScreenState,
        preciseMode: PreciseMode,
        chars: Vec<Char>
    },
    SetBG {
        idx: u8
    },
    SetFG {
        idx: u8
    },
    SetPalette {
        r: u8,
        g: u8,
        b: u8,
        idx: u8
    },
    SetResolution {
        resolution: ScreenResolution
    },
    SetChars {
        x: u8,
        y: u8,
        chars: String,
        isVertical: bool
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
        charCode: u32
    },
    SetScreenState {
        screenState: ScreenState
    },
    SetPrecise {
        preciseMode: PreciseMode
    },
    Fetch,
    EventTouch {
        x: u8,
        y: u8,
        button: u8
    },
    EventDrag {
        x: u8,
        y: u8,
        button: u8
    },
    EventDrop {
        x: u8,
        y: u8,
        button: u8
    },
    EventScroll {
        x: u8,
        y: u8,
        direction: Direction,
        delta: u8
    },
    EventKeyDown {
        charCode: u32,
        lwjglCode: u32
    },
    EventKeyUp {
        charCode: u32,
        lwjglCode: u32
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
