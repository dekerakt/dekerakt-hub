use mio;

use graphics;
use message;

pub struct Store(Vec<Vertex>);

impl Store {
    pub fn new() -> Store {
        Store(Vec::new())
    }

    pub fn find_vertex_with_user(&self, user: String) -> Option<&Vertex> {
        for vertex in &self.0 {
            if *vertex.user == user {
                return Some(vertex);
            }
        }
        return None
    }

    pub fn find_vertex_with_user_mut(&mut self, user: String) -> Option<&mut Vertex> {
        for vertex in &mut self.0 {
            if *vertex.user == user {
                return Some(vertex);
            }
        }
        return None
    }
}

pub struct Vertex {
    pub opencomputers: Option<mio::Token>,
    pub external: Option<mio::Token>,
    pub palette: [graphics::Color; 16],
    pub mode: message::ConnectionMode,
    pub user: String,
    pub password: String,
    pub screen_state: graphics::ScreenState,
    pub precise_mode: graphics::PreciseMode,
    pub canvas: graphics::Canvas,
}
