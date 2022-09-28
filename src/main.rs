use glutin_window::GlutinWindow;
use piston::event_loop::{EventSettings, Events};
use piston::{EventLoop, RenderEvent, WindowSettings};
use opengl_graphics::{OpenGL, GlGraphics};
use graphics::{clear};

pub use crate::chess_controller::ChessController;
pub use crate::chess_graphcis::{ChessGraphics, ChessGraphicsSettings};

mod chess_controller;
mod chess_graphcis;

fn main() {
    let opengl = OpenGL::V3_2;
    let settings = WindowSettings::new("Sudoku", (800, 600))
        .exit_on_esc(true)
        .graphics_api(opengl)
        .vsync(true);

    let mut window: GlutinWindow =
        settings.build().expect("Could not create window");

    let mut events = Events::new(EventSettings::new().lazy(true));
    let mut gl = GlGraphics::new(opengl);

    let mut gameboard_controller = ChessController::new();
    let gameboard_view_settings = ChessGraphicsSettings::new();
    let gameboard_view = ChessGraphics::new(gameboard_view_settings);

    while let Some(e) = events.next(&mut window) {
        gameboard_controller.event(
            gameboard_view.settings.offset,
            gameboard_view.settings.size,
            gameboard_view.settings.square_amount,
            &e,
        );
        if let Some(args) = e.render_args() {
            gl.draw(args.viewport(), |c, g| {
                clear([0.4, 0.4, 0.4, 1.0], g);
                gameboard_view.draw(&gameboard_controller, &c, g);
            });
        }
    }
}