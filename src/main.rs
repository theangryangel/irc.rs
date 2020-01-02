mod wire;
mod tags;

use std::io::prelude::*;
use std::net::{TcpStream};
use std::io::{BufWriter, BufReader};

fn main() {

    match TcpStream::connect("chat.freenode.net:6667") {
        Ok(mut stream) => {
            println!("Successfully connected");

            let cap = wire::RawMsg{source: None, tags: None, command: "CAP".to_string(), params: vec!["LS".to_string(), "302".to_string()]};
            stream.write(format!("{}\r\n", &cap.to_string()).as_bytes());
 
            let user = wire::RawMsg{source: None, tags: None, command: "USER".to_string(), params: vec![
                "MrBotMcBotFace".to_string(),
                "0".to_string(),
                "*".to_string(),
                "Karl".to_string()
            ]};
            stream.write(format!("{}\r\n", &user.to_string()).as_bytes());

            let nick = wire::RawMsg{source: None, tags: None, command: "NICK".to_string(), params: vec!["MrBotMcBotFace".to_string()]};
            stream.write(format!("{}\r\n", &nick.to_string()).as_bytes());

            // TODO this should wait until after we get the CAP resp
            let cap_end = wire::RawMsg{source: None, tags: None, command: "CAP".to_string(), params: vec!["END".to_string()]};
            stream.write(format!("{}\r\n", &cap_end.to_string()).as_bytes());

            let mut reader = BufReader::new(&stream);
            loop {
                let mut raw: String = String::new();

                match reader.read_line(&mut raw) {
                    Err(e) => {
                        println!("Error? {}", e);
                        break;
                    }
                    Ok(0) => {
                        println!("EOF?");
                        break;
                    },
                    Ok(len) => {
                        println!("Got RAW: {}, Len={}", raw.trim(), len);
                        let msg = wire::RawMsg::from_string(raw);
                        println!("Decoded as command={}", msg.command);

                        match msg.command.as_ref() {
                            "PING" => {
                                let pong = wire::RawMsg{
                                    source: None, 
                                    tags: None, 
                                    command: "PONG".to_string(), 
                                    params: vec![]
                                };
                                reader.get_mut().write(format!("{}\r\n", &pong.to_string()).as_bytes());
                            },
                            _ => continue
                        }

                    },
                }
            }

        },
        Err(e) => {
            println!("Failed to connect: {}", e);
        }

    }

    println!("Quit!");

}
