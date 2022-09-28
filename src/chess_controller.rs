//! Chess controller.

use piston::GenericEvent;
use piston::input::{Button, MouseButton};
use chess_lib_temp_clone::ChessBoard;

/// Handles events for Chess.
pub struct ChessController {
    /// Stores the chess board.
    pub chess_engine: ChessBoard,
    /// Stores the selected square.
    pub selected_square: Option<[u8; 2]>,
    /// Stores the hovered square.
    pub hovered_square: Option<[u8; 2]>,
    /// Stores the mouse coords.
    mouse_coords: [f64; 2]
}

impl ChessController {
    /// Creates a new chess board controller.
    pub fn new() -> ChessController {
        ChessController {
            chess_engine: ChessBoard::init_position(),
            selected_square: None,
            hovered_square: None,
            mouse_coords: [0.0; 2]
        }
    }

    /// Handles events.
    pub fn event<E: GenericEvent>(&mut self, offset: [f64; 2], size: f64, square_amount: f64, e: &E) {
        if let Some(pos) = e.mouse_cursor_args() {
            self.mouse_coords = pos;
            // println!("{:?}", self.mouse_pos)
        }

        let x = self.mouse_coords[0] - offset[0];
        let y = self.mouse_coords[1] - offset[1];
        // Check that coordinates are inside board boundaries.
        if x >= 0.0 && x < size && y >= 0.0 && y < size {
            // Compute the cell position.
            let (coords_x, coords_y) = ((x / size * square_amount) as u8,
                                        (y / size * square_amount) as u8);
            self.hovered_square = Some([coords_x, coords_y]);
            if let Some(Button::Mouse(MouseButton::Left)) = e.press_args() {
                self.selected_square = Some([coords_x, coords_y]);
            }
        }
    }
}