use mio::Token;

pub const SERVER_TOKEN: Token = Token(::std::usize::MAX - 10);

pub const EVENTS_CAPACITY: usize = 1024;
pub const CONNECTIONS_CAPACITY: usize = 8192;

pub const CONNECTION_READ_BUF_CAPACITY: usize = 4096;
pub const CONNECTION_READ_BUF_MAX_CAPACITY: usize = 1048576;
pub const CONNECTION_READ_CHUNK_SIZE: usize = 4096;
pub const CONNECTION_WRITE_BUF_CAPACITY: usize = 4096;

// TODO: load from file/CLI

