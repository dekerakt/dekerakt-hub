use std::ops::{Deref, DerefMut};

use mio::Token;
use slab::Slab;

use config::*;
use server::{Client, ClientData};

#[derive(Debug)]
pub struct Clients(Slab<Client, Token>);

impl Clients {
    pub fn new() -> Clients {
        Clients(Slab::with_capacity(CONNECTIONS_CAPACITY))
    }

    pub fn is_username_unique(&self, new_username: &str) -> bool {
        for ref item in self.iter() {
            if let Some(ClientData { ref username, .. }) = item.data {
                if new_username == username {
                    return false;
                }
            }
        }

        true
    }
}

impl Deref for Clients {
    type Target = Slab<Client, Token>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Clients {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

