use mio::Token;

error_chain! {
    types {
        Error, ErrorKind, ResultExt, Result;
    }

    foreign_links {
        Io(::std::io::Error);
        Utf8(::std::str::Utf8Error);
    }

    errors {
        UnknownOpcode(c: u8) {
            description("unknown opcode")
            display("unknown opcode: {}", c)
        }

        UnknownHandshakeStatus(c: u8) {
            description("unknown handshake status")
            display("unknown handshake status: {}", c)
        }

        UnknownConnectionStatus(c: u8) {
            description("unknown connection status")
            display("unknown connection status: {}", c)
        }

        InvalidToken(t: Token) {
            description("invalid token")
            display("invalid token: {}", t.0)
        }

        SpaceNeeded(n: usize) {
            description("space needed")
            display("space needed: {}", n)
        }

        BufferOverflow {
            description("buffer overflow")
            display("buffer overflow")
        }
    }
}
