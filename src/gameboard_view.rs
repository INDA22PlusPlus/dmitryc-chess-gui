//! Gameboard view.

use graphics::types::Color;
use graphics::{Context, Graphics, Line, Rectangle};


use crate::gameboard_controller::GameboardController;

/// Stores gameboard view settings.
pub struct GameboardViewSettings {
    /// Position from left-top corner.
    pub offset: [f64; 2],
    /// Size of gameboard along horizontal and vertical edge.
    pub size: f64,
    /// Amount of squares
    pub square_amount: f64,
    /// Amount of squares
    pub square_side: f64,
    /// White color.
    pub white_color: Color,
    /// Black color.
    pub black_color: Color,
}

impl GameboardViewSettings {
    /// Creates new gameboard view settings.
    pub fn new() -> GameboardViewSettings {
        let size = 500.0;
        let square_amount = 8.0;
        let square_side = size / square_amount;
        GameboardViewSettings {
            offset: [50.0; 2],
            size,
            square_amount,
            square_side,
            white_color: [1.0, 1.0, 0.9, 1.0],
            black_color: [0.30, 0.15, 0.15, 1.0],
        }
    }
}

/// Stores visual information about a gameboard.
pub struct GameboardView {
    /// Stores gameboard view settings.
    pub settings: GameboardViewSettings,
}

impl GameboardView {
    /// Creates a new gameboard view.
    pub fn new(settings: GameboardViewSettings) -> GameboardView {
        GameboardView {
            settings: settings,
        }
    }

    /// Draw gameboard.
    pub fn draw<G: Graphics>(
        &self,
        controller: &GameboardController,
        c: &Context,
        g: &mut G,
    ) {
        let ref settings = self.settings;

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
            }
        }
    }
}