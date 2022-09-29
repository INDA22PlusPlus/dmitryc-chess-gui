//! Chess board view.

use std::path::Path;
use graphics::types::Color;
use graphics::{Context, Graphics, Line, Rectangle, Text, Image};
use opengl_graphics::{Texture, TextureSettings};

use dynchess_lib::ChessPiece;

use crate::chess_controller::ChessController;

/// Stores chess board view settings.
pub struct ChessGraphicsSettings {
    /// Position from left-top corner.
    pub offset: [f64; 2],
    /// Size of chess board along horizontal and vertical edge.
    pub size: f64,
    /// Amount of squares
    pub square_amount: f64,
    /// Amount of squares
    pub square_side: f64,
    /// White color.
    pub white_color: Color,
    /// Black color.
    pub black_color: Color,
    /// Selected square color
    pub selected_square_background_color: Color,
    /// Hovered square color
    pub hovered_square_background_color: Color,
}

impl ChessGraphicsSettings {
    /// Creates new chess board view settings.
    pub fn new() -> ChessGraphicsSettings {
        let size = 500.0;
        let square_amount = 8.0;
        let square_side = size / square_amount;
        ChessGraphicsSettings {
            offset: [50.0; 2],
            size,
            square_amount,
            square_side,
            white_color: [1.0, 1.0, 0.9, 1.0],
            black_color: [0.30, 0.15, 0.15, 1.0],
            selected_square_background_color: [0.3, 0.3, 0.5, 1.0],
            hovered_square_background_color: [0.7, 0.75, 0.9, 1.0],
        }
    }

}

/// Stores visual information about a chess board.
pub struct ChessGraphics {
    /// Stores chess board view settings.
    pub settings: ChessGraphicsSettings,
}

impl ChessGraphics {
    /// Creates a new chess board view.
    pub fn new(settings: ChessGraphicsSettings) -> ChessGraphics {
        ChessGraphics {
            settings: settings,
        }
    }

    /// Draw chess board.
    pub fn draw<G: Graphics<Texture = Texture>>(
        &self,
        controller: &ChessController,
        c: &Context,
        g: &mut G,
    ) {
        let ref settings = self.settings;
        let ref board = controller.chess_engine.get_board();
        let w_pawn = Texture::from_path(Path::new("sprites/w_pawn.png"),
                                        &TextureSettings::new()).unwrap();

        // TODO: Ranks and flanks
        // let mut glyphs = Glyphs::from_bytes(
        //     font,
        //     window.create_texture_context(),
        //     TextureSettings::new(),
        // );

        // Text::new_color([0.0, 0.0, 0.0, 1.0], 32).draw(
        //     "test",
        //     &mut glyphs,
        //     &c.draw_state,
        //     c.transform,
        //     g
        // );

        // Draw board
        for x in 0..=7 {
            for y in 0..=7 {
                let square_rect = [
                    settings.offset[0] + settings.square_side * x as f64,
                    settings.offset[1] + settings.square_side * y as f64,
                    settings.square_side,
                    settings.square_side,
                ];

                let mut color = settings.white_color;
                if x % 2 + y % 2 == 1 {
                    color = settings.black_color;
                }

                Rectangle::new(color).draw(
                    square_rect,
                    &c.draw_state,
                    c.transform,
                    g,
                );

                // TODO: Fix duplication
                if let Some(square) = controller.hovered_square {
                    let square_start_coords = [
                        square[0] as f64 * settings.square_side,
                        square[1] as f64 * settings.square_side,
                    ];

                    let selected_square_rect = [
                        settings.offset[0] + square_start_coords[0],
                        settings.offset[1] + square_start_coords[1],
                        settings.square_side,
                        settings.square_side,
                    ];

                    // TODO: Fix duplication
                    Rectangle::new(settings.hovered_square_background_color).draw(
                        selected_square_rect,
                        &c.draw_state,
                        c.transform,
                        g,
                    );
                }

                if let Some(square) = controller.selected_square {
                    let square_start_coords = [
                        square[0] as f64 * settings.square_side,
                        square[1] as f64 * settings.square_side,
                    ];

                    let selected_square_rect = [
                        settings.offset[0] + square_start_coords[0],
                        settings.offset[1] + square_start_coords[1],
                        settings.square_side,
                        settings.square_side,
                    ];

                    Rectangle::new(settings.selected_square_background_color).draw(
                        selected_square_rect,
                        &c.draw_state,
                        c.transform,
                        g,
                    );
                }

                let piece = board[(x + y * 8) as usize];
                // println!("{:?} {}", piece, x + y * 8);
                if !(piece == ChessPiece::Empty) {
                    // let w_pawn = Texture::from_path(Path::new("sprites/w_pawn.png"),
                    //                                 &TextureSettings::new()).unwrap();
                    let square_drawable = Image::new().rect(square_rect);
                    square_drawable.draw(&w_pawn, &c.draw_state, c.transform, g);
                }
            }
        }
    }
}