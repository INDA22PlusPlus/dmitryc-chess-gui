use std::{boxed, io::{Read, Write}, net::{TcpListener, TcpStream}};
use dynchess_lib::ChessBoard;
use prost::Message;
use crate::networking_protobuf;
use crate::networking_protobuf::{
    c2s_message,
    C2sConnectRequest,
    C2sMessage,
    s2c_message,
    S2cConnectAck,
    S2cMessage,
    Move
};

#[derive(PartialEq, Debug)]
pub enum State {
    Playing,
    WaitingForOpponent,
}

#[derive(PartialEq, Clone)]
pub enum ConnectionType {
    Host(S2cMessage),
    Client(C2sMessage),
}

pub struct Networking {
    // Logic
    // pub from: u8,
    // pub to: u8,
    pub state: State,

    // Networking
    pub socket: TcpStream,

    pub connection: ConnectionType,
    pub game_id: u64,
}

impl Networking {
    pub(crate) fn new() -> Networking {
        let game_id = 1;
        // A stream and a boolean indicating whether or not the program is a host or a client
        let (stream, is_client, mut connection_type) = {
            let mut args = std::env::args();
            // Skip path to program
            args.next();

            // Get first argument after path to program
            let host_or_client = args
                .next()
                .expect("Expected arguments: --host or --client 'ip'");

            match host_or_client.as_str() {
                // If the program is running as host we listen on port 8080 until we get a
                // connection then we return the stream.
                "--host" => {
                    let ip = args.next().expect("Expected ip address after --client");
                    let listener = TcpListener::bind(ip).unwrap();
                    (listener.incoming().next().unwrap().unwrap(), false, ConnectionType::Host(
                        S2cMessage {
                            msg: Some(s2c_message::Msg::ConnectAck(S2cConnectAck {
                                success: false,
                                game_id: Some(game_id),
                                starting_position: None,
                                client_is_white: None
                            }))
                        })
                    )
                }
                // If the program is running as a client we connect to the specified IP address and
                // return the stream.
                "--client" => {
                    let ip = args.next().expect("Expected ip address after --client");
                    let stream = TcpStream::connect(ip).expect("Failed to connect to host");
                    (stream, true, ConnectionType::Client(
                        C2sMessage{
                            msg: Some(c2s_message::Msg::ConnectRequest(C2sConnectRequest{
                                game_id,
                                spectate: false
                            }))
                        })
                    )
                }
                // Only --host and --client are valid arguments
                _ => panic!("Unknown command: {}", host_or_client),
            }
        };

        // Set TcpStream to non blocking so that we can do networking in the update thread
        stream
            .set_nonblocking(true)
            .expect("Failed to set stream to non blocking");

        Networking {
            // from: if client { 0 } else { 63 },
            // to: if client { 63 } else { 0 },
            // Host starts playing and the client waits
            state: if is_client {
                State::WaitingForOpponent
            } else {
                State::Playing
            },
            socket: stream,
            connection: connection_type,
            game_id,
        }
    }

    /// Checks if a move packet is available in returns the new positions otherwise it returns none
    pub fn receive_packet(&mut self) -> Option<[u8; 2]> {
        // println!("Received packet");
        let mut buf = [0u8; 512];
        let mut coords = None;
        let (packet_length, packet_ok) = match self.socket.read(&mut buf) {
            Ok(length) => (length, true),
            Err(e) => (0, false)
        };

        if packet_ok {
            // println!("Packet is ok");
            match &self.connection {
                ConnectionType::Host(host) => {
                    let packet_decoded = C2sMessage::decode(&buf[0..packet_length]);
                    match packet_decoded.clone().unwrap().msg.unwrap() {
                        c2s_message::Msg::Move(move_packet) => {
                            self.connection = ConnectionType::Host(
                                S2cMessage{ msg: None }
                            );
                            coords = Some([
                                move_packet.from_square as u8,
                                move_packet.to_square as u8
                            ]);
                            self.state = State::Playing;
                            // println!("changed state, returned new coords");
                        }
                        c2s_message::Msg::ConnectRequest(_) => {
                            self.connection = ConnectionType::Host(
                                S2cMessage {
                                    msg: Some(s2c_message::Msg::ConnectAck(S2cConnectAck {
                                        success: true,
                                        game_id: Some(self.game_id),
                                        starting_position: None,
                                        client_is_white: None
                                    }))
                                }
                            );
                            self.send_packet(None);
                            self.connection = ConnectionType::Host(
                                S2cMessage{ msg: None }
                            );
                        }
                    }
                    // println!("{:?}", packet_decoded);
                }
                ConnectionType::Client(client) => {
                    let packet_decoded = S2cMessage::decode(&buf[0..packet_length]);

                    match packet_decoded.clone().unwrap().msg.unwrap() {
                        s2c_message::Msg::Move(move_packet) => {
                            coords = Some([
                                move_packet.from_square as u8,
                                move_packet.to_square as u8
                            ]);
                            self.state = State::Playing;
                            // println!("changed state, returned new coords");
                        }
                        s2c_message::Msg::ConnectAck(con_ack) => {
                            if con_ack.success {
                                // println!("successful connection");
                                self.connection = ConnectionType::Client(
                                    C2sMessage{ msg: None }
                                );
                            }
                        }
                        s2c_message::Msg::MoveAck(_) => {}
                    }

                    // println!("{:?}", packet_decoded);
                }
            };
        }

        coords
    }

    /// Sends a move packet of the current position and sets the state to waiting
    pub fn send_packet(&mut self, coords: Option<[u8; 2]>) {
        let mut buf = match self.connection.clone() {
            ConnectionType::Host(host) => {
                let mut new_host= host;
                if let Some(coords_vec) = coords {
                    new_host = S2cMessage {
                        msg: Some(s2c_message::Msg::Move(Move {
                            from_square: coords_vec[0] as u32,
                            to_square: coords_vec[1] as u32,
                            promotion: None
                        }))
                    };
                    // println!("changed cords to send them {:?}", coords_vec);
                }

                prost::Message::encode_to_vec(&new_host)
            }
            ConnectionType::Client(client) => {
                let mut new_client= client;
                if let Some(coords_vec) = coords {
                    new_client = C2sMessage {
                        msg: Some(c2s_message::Msg::Move(Move {
                            from_square: coords_vec[0] as u32,
                            to_square: coords_vec[1] as u32,
                            promotion: None
                        }))
                    };
                    // println!("changed cords to send them {:?}", coords_vec);
                }

                prost::Message::encode_to_vec(&new_client)
            }
        };

        self.socket
            .write(&buf)
            .expect("Failed to send move packet");
        // self.state = State::WaitingForOpponent;
        // println!("Packet send: {:?}", buf);
    }

    pub fn update(&mut self) {
        match self.state {
            State::Playing => {}
            State::WaitingForOpponent => {
                // If we received at move packet we first set the enemy pos to the received
                // position and then set the state to playing
                if let Some(pos) = self.receive_packet() {
                    self.state = State::Playing;
                    // self.enemy_pos = pos;
                }
            }
        }
    }
}