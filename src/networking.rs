use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
};
use dynchess_lib::ChessBoard;
use crate::networking_protobuf::{c2s_message, C2sConnectRequest, C2sMessage, s2c_message, S2cConnectAck, S2cMessage};
use crate::networking_protobuf::c2s_message::Msg::ConnectRequest;
use crate::networking_protobuf::s2c_message::Msg::ConnectAck;
use prost::Message;
use polling::{Event, Poller};

#[derive(PartialEq)]
pub enum State {
    Playing,
    WaitingForOpponent,
}

#[derive(PartialEq, Clone)]
pub enum ConnectionType {
    host(S2cMessage),
    client(C2sMessage),
}

pub struct Networking {
    pub state: State,

    // Networking
    pub socket: TcpStream,
    pub poller: Poller,

    connection: ConnectionType,

    pub game_id: u64,
    pub poller_key: usize,
}

impl Networking {
    pub(crate) fn new() -> Networking {
        let game_id = 1 as u64;
        let poller_key = game_id.clone() as usize;
        // A stream and a boolean indicating whether or not the program is a host or a client
        let (stream, client, connection_type) = {
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
                    let listener = TcpListener::bind("127.0.0.1:1337").unwrap();
                    (listener.incoming().next().unwrap().unwrap(), false, ConnectionType::host(
                        S2cMessage {
                            msg: Some(ConnectAck(S2cConnectAck {
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
                    (stream, true, ConnectionType::client(C2sMessage{
                        msg: Some(ConnectRequest(C2sConnectRequest{
                            game_id,
                            spectate: false
                        }))
                    }))
                }
                // Only --host and --client are valid arguments
                _ => panic!("Unknown command: {}", host_or_client),
            }
        };

        // Set TcpStream to non blocking so that we can do networking in the update thread
        stream
            .set_nonblocking(true)
            .expect("Failed to set stream to non blocking");

        let poller = Poller::new()
            .expect("Couldn't create a Poller");
        poller.add(&stream, Event::readable(poller_key))
            .expect("Couldn't add to poller");

        Networking {
            state: if client {
                State::WaitingForOpponent
            } else {
                State::Playing
            },
            socket: stream,
            poller,
            connection: connection_type,
            game_id,
            poller_key,
        }
    }

    /// Checks if a move packet is available in returns the new positions otherwise it returns none
    pub fn receive_move_packet(&mut self) -> Option<[u8; 512]> {
        let mut events = vec![];
        self.poller.wait(&mut events, None).expect("Couldn't receive events with None timeout");

        let mut packet = None;
        for ev in &events {
            if ev.key == self.poller_key {
                // Perform a non-blocking accept operation.
                self.socket.accept()
                    .except("Couldn't accept a stream");
                // Set interest in the next readability event.
                self.poller.modify(&self.socket, Event::readable(self.poller_key))
                    .expect("Couldn't modify the poller");

                // packet = match self.stream.read(&mut packet) {
                //     Ok(_) => Some(packet),
                //     Err(e) => match e.kind() {
                //         std::io::ErrorKind::WouldBlock => None,
                //         _ => panic!("Error: {}", e),
                //     },
                // };
            }
        }

        packet
    }

    /// Sends a move packet of the current position and sets the state to waiting
    pub fn send_move_packet(&mut self) {
        let mut buf = match self.connection.clone() {
            ConnectionType::host(host) => {
                host.encode_to_vec()
            }
            ConnectionType::client(client) => {
                client.encode_to_vec()
            }
        };

        self.poller
            .write(&buf)
            .expect("Failed to send move packet");
        self.state = State::WaitingForOpponent;
    }

    pub fn update(&mut self) {
        match self.state {
            State::Playing => {}
            State::WaitingForOpponent => {
                // If we received at move packet we first set the enemy pos to the received
                // position and then set the state to playing
                if let Some(pos) = self.receive_move_packet() {
                    self.state = State::Playing;
                    // self.enemy_pos = pos;
                }
            }
        }
    }
}