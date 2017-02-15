pub type Store = Vec<Vertex>;

struct Vertex {
    opencomputers: &mio::Token,
    external: &mio::Token,
    palette: [Graphics::Color; 16],
    mode: Messages::ConnectionMode,
    user: String,
    password: String,
    screen_state: Graphics::ScreenState,
    precise_mode: Graphics::PreciseMode,
    canvas: Graphics::Canvas,
}
