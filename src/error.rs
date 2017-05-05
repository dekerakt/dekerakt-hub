use mio::Token;

error_chain! {
    types {
        Error, ErrorKind, ResultExt, Result;
    }

    foreign_links {
        Io(::std::io::Error);
        Utf8(::std::string::FromUtf8Error);
    }

    errors {
        UnknownOpcode(b: u8) {
            description("unknown opcode")
            display("unknown opcode ({})", b)
        }

        UnknownAuthStatus(b: u8) {
            description("unknown auth status")
            display("unknown auth status ({})", b)
        }

        UnknownErrorCode(b: u8) {
            description("unknown error code")
            display("unknown error code ({})", b)
        }

        InvalidToken(t: Token) {
            description("invalid token")
            display("invalid token ({:?})", t)
        }
    }
}

