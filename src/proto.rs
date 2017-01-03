use std::{io, str};

use tokio_core::io::{Io, Framed};
use tokio_proto::pipeline::ServerProto;

use codec::Codec;

pub struct Proto;

impl<T: Io + 'static> ServerProto<T> for Proto {
    type Request = String;
    type Response = String;
    type Error = io::Error;
    type Transport = Framed<T, Codec>;
    type BindTransport = io::Result<Framed<T, Codec>>;

    fn bind_transport(&self, io: T) -> io::Result<Framed<T, Codec>> {
        Ok(io.framed(Codec))
    }
}

