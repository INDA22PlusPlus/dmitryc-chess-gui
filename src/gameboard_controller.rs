//! Chess controller.

use piston::GenericEvent;

use dynchess_lib::ChessBoard;

/// Handles events for Chess.
pub struct ChessController {
    /// Stores the chess board state.
    pub chess_board: ChessBoard,
}

impl ChessController {
    /// Creates a new chess board controller.
    pub fn new() -> ChessController {
        ChessController {
            chess_board: ChessBoard::init_position(),
        }
    }

    /// Handles events.
    pub fn event<E: GenericEvent>(&mut self, e: &E) {

    }
}