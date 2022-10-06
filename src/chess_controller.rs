//! Chess controller.

use std::borrow::Borrow;
use std::num::IntErrorKind::Empty;
use piston::GenericEvent;
use piston::input::{Button, MouseButton};
use dynchess_lib::{ChessBoard, ChessPiece};
use crate::networking::{ConnectionType, Networking, State};
use crate::networking_protobuf::{S2cConnectAck, S2cMessage, s2c_message, c2s_message};

/// Handles events for Chess.
pub struct ChessController {
    /// Stores the chess board.
    pub chess_engine: ChessBoard,
    /// Stores the selected square.
    pub selected_square: Option<[u8; 2]>,
    /// Stores the hovered square.
    pub hovered_square: Option<[u8; 2]>,
    /// Stores the mouse coords.
    mouse_coords: [f64; 2],
    /// Networking
    pub networking: Networking,
}

impl ChessController {
    /// Creates a new chess board controller.
    pub fn new() -> ChessController {
        let chess_controller = ChessController {
            chess_engine: ChessBoard::init_position(),
            selected_square: None,
            hovered_square: None,
            mouse_coords: [0.0; 2],
            networking: Networking::new(),
        };
        chess_controller
    }

    /// Handles events.
    pub fn event<E: GenericEvent>(&mut self, offset: [f64; 2], size: f64, square_amount: f64, e: &E) {
        // Initial connection
        // println!("looping event");
        match self.networking.connection.clone() {
            ConnectionType::Host(host) => {
                if host.msg.is_some(){
                    match host.msg.unwrap() {
                        // s2c_message::Msg::Move(_) => {}
                        s2c_message::Msg::ConnectAck(_) => {
                            self.networking.receive_packet();
                            // self.networking.send_packet(0, 0);
                            return
                        }
                        // s2c_message::MsgMsg::MoveAck(_) => {}
                        _ => {}
                    }
                }
            }
            ConnectionType::Client(client) => {
                if client.msg.is_some() {
                    match client.msg.unwrap() {
                        // c2s_message::Msg::Move(_) => {}
                        c2s_message::Msg::ConnectRequest(_) => {
                            self.networking.receive_packet();
                            self.networking.send_packet(0, 0);
                            return
                        }
                        _ => {}
                    }
                }
            }
        };

        // When connected
        // println!("connected, state: {:?}", self.networking.state);
        match self.networking.state {
            State::Playing => {
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
                        let to_coords_u8 = coords_x + (7 - coords_y) * 8;
                        if self.selected_square.is_some(){
                            let selected_coords = self.selected_square.unwrap();
                            let selected_coords_to_u8 = selected_coords[0] + (7 - selected_coords[1]) * 8;
                            // println!("{:?}, {:?}", selected_coords_to_u8, to_coords_u8);
                            self.chess_engine.drag(selected_coords_to_u8, to_coords_u8);
                            self.selected_square = None;

                            // println!("try sending packet");
                            self.networking.send_packet(selected_coords_to_u8, to_coords_u8);
                            // println!("sent packet");
                            self.networking.state = State::WaitingForOpponent;
                            // println!("changing state {:?}", self.networking.state);
                        }
                        else {
                            if !(self.chess_engine.get_piece(to_coords_u8) == ChessPiece::Empty) {
                                self.selected_square = Some([coords_x, coords_y])
                            }
                        }
                    }
                }
            }
            State::WaitingForOpponent => {
                if let Some(buf) = self.networking.receive_packet() {
                    // self.networking.state = State::Playing;

                    self.chess_engine.drag(buf[0], buf[1]);
                }
            }
        }
    }
}