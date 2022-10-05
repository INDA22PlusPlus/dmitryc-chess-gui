use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
};

#[derive(PartialEq)]
pub enum State {
    Playing,
    WaitingForOpponent,
}

pub struct Networking {
    // Logic
    player_pos: [u8; 2],
    enemy_pos: [u8; 2],
    pub state: State,

    // Networking
    pub stream: TcpStream,
}

impl Networking {
    pub(crate) fn new() -> Networking {
        // A stream and a boolean indicating whether or not the program is a host or a client
        let (stream, client) = {
            let mut args = std::env::args();
            // Skip path to program
            let _ = args.next();

            // Get first argument after path to program
            let host_or_client = args
                .next()
                .expect("Expected arguments: --host or --client 'ip'");

            match host_or_client.as_str() {
                // If the program is running as host we listen on port 8080 until we get a
                // connection then we return the stream.
                "--host" => {
                    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
                    (listener.incoming().next().unwrap().unwrap(), false)
                }
                // If the program is running as a client we connect to the specified IP address and
                // return the stream.
                "--client" => {
                    let ip = args.next().expect("Expected ip address after --client");
                    let stream = TcpStream::connect(ip).expect("Failed to connect to host");
                    (stream, true)
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
            player_pos: if client { [7 as u8; 2] } else { [0 as u8; 2] },
            enemy_pos: if client { [0 as u8; 2] } else { [7 as u8; 2] },
            // Host starts playing and the client waits
            state: if client {
                State::WaitingForOpponent
            } else {
                State::Playing
            },
            stream,
        }
    }

    /// Checks if a move packet is available in returns the new positions otherwise it returns none
    pub fn receive_move_packet(&mut self) -> Option<[u8; 2]> {
        let mut buf = [0u8; 2];
        match self.stream.read(&mut buf) {
            Ok(_) => Some(buf),
            Err(e) => match e.kind() {
                std::io::ErrorKind::WouldBlock => None,
                _ => panic!("Error: {}", e),
            },
        }
    }

    /// Sends a move packet of the current position and sets the state to waiting
    pub fn send_move_packet(&mut self) {
        self.stream
            .write(&mut self.player_pos)
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
                    self.enemy_pos = pos;
                }
            }
        }
    }
}