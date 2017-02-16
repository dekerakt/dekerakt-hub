use mio;

use graphics;
use message;

pub type Store = Vec<Vertex>;

struct Vertex {
    opencomputers: mio::Token,
    external: mio::Token,
    palette: [graphics::Color; 16],
    mode: message::ConnectionMode,
    user: String,
    password: String,
    screen_state: graphics::ScreenState,
    precise_mode: graphics::PreciseMode,
    canvas: graphics::Canvas,
}
