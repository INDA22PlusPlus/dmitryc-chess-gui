use glutin_window::GlutinWindow;
use piston::event_loop::{EventSettings, Events};
use piston::{EventLoop, RenderEvent, WindowSettings};
use opengl_graphics::{OpenGL, GlGraphics};
use graphics::{clear};

pub use crate::chess_controller::ChessController;
pub use crate::chess_graphics::{ChessGraphics, ChessGraphicsSettings};
pub use crate::networking::Networking;

mod chess_controller;
mod chess_graphics;
mod networking;
mod networking_protobuf;

fn main() {
    let opengl = OpenGL::V3_2;

    let mut args = std::env::args();
    // Skip path to program
    args.next();

    // Get first argument after path to program
    let host_or_client = args
        .next()
        .expect("Expected arguments: --host or --client 'ip'");


    let name = match host_or_client.as_str() {
        "--host" => {
            "Chess - Host"
        }
        "--client" => {
            "Chess - Client"
        }
        // Only --host and --client are valid arguments
        _ => panic!("Unknown command: {}", host_or_client),
    };

    let settings = WindowSettings::new(name, (600, 600))
        .exit_on_esc(true)
        .graphics_api(opengl)
        .vsync(true);

    let mut window: GlutinWindow =
        settings.build().expect("Could not create window");

    let mut events = Events::new(EventSettings::new().lazy(true));
    let mut gl = GlGraphics::new(opengl);

    let mut chess_controller = ChessController::new();
    let chess_view_settings = ChessGraphicsSettings::new();
    let chess_view = ChessGraphics::new(chess_view_settings);

    while let Some(e) = events.next(&mut window) {
        chess_controller.event(
            chess_view.settings.offset,
            chess_view.settings.size,
            chess_view.settings.square_amount,
            &e,
        );
        if let Some(args) = e.render_args() {
            gl.draw(args.viewport(), |c, g| {
                clear([0.4, 0.4, 0.4, 1.0], g);
                chess_view.draw(&chess_controller, &c, g);
            });
        }
    }
}