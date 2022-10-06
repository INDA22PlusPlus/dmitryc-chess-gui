use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
};
use dynchess_lib::ChessBoard;
use crate::networking_protobuf::{c2s_message, C2sConnectRequest, C2sMessage, s2c_message, S2cConnectAck, S2cMessage};
use crate::networking_protobuf::c2s_message::Msg::ConnectRequest;
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
    // Logic
    // pub from: u8,
    // pub to: u8,
    pub state: State,

    // Networking
    pub poller: Poller,

    connection: ConnectionType,

    pub game_id: u64,
    pub poller_key: u64,
}

impl Networking {
    pub(crate) fn new() -> Networking {
        let game_id = 1 as u64;
        let poller_key = game_id.clone();
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
                    (stream, true, ConnectionType::client(C2sMessage{
                        msg: Some(c2s_message::Msg::ConnectRequest(C2sConnectRequest{
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

        Networking {
            // from: if client { 0 } else { 63 },
            // to: if client { 63 } else { 0 },
            // Host starts playing and the client waits
            state: if client {
                State::WaitingForOpponent
            } else {
                State::Playing
            },
            poller,
            connection: connection_type,
            game_id,
            poller_key,
        }
    }

    /// Checks if a move packet is available in returns the new positions otherwise it returns none
    pub fn receive_move_packet(&mut self) -> Option<[u8; 512]> {
        events.clear();
        poller.wait(&mut events, None)?;

        let packet;
        for ev in &events {
            if ev.key == key {
                // Perform a non-blocking accept operation.
                socket.accept()?;
                // Set interest in the next readability event.
                poller.modify(&socket, Event::readable(self.poller_key))?;

                let mut buf = [0u8; 512];
                let packet  = match self.poller.read(&mut buf) {
                    Ok(_) => Some(buf),
                    Err(e) => match e.kind() {
                        std::io::ErrorKind::WouldBlock => None,
                        _ => panic!("Error: {}", e),
                    },
                };
            }
        }

        packet
    }

    /// Sends a move packet of the current position and sets the state to waiting
    pub fn send_move_packet(&mut self) {
        // self.from = from;
        // self.to = to;
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
        // println!("Packet send: {:?}", buf);
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