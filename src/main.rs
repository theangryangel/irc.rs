mod wire;
mod tags;
mod codec;

use tokio::net::TcpStream;
use tokio_util::codec::Framed;
use tokio::stream::StreamExt;
use futures::{FutureExt, SinkExt};
use config::Config;
use std::collections::HashMap;

#[tokio::main]
pub async fn main() {

    let mut settings = Config::default();
    settings
        // Add in `./Settings.toml`
        .merge(config::File::with_name("Settings")).unwrap()
        // Add in settings from the environment (with a prefix of APP)
        // Eg.. `APP_DEBUG=1 ./target/app` would set the `debug` key
        .merge(config::Environment::with_prefix("APP")).unwrap();

    let stream = TcpStream::connect(settings.get_str("server").unwrap()).await.unwrap();

    let mut transport = Framed::new(stream, codec::IrcCodec::new());

    let cap = wire::RawMsg{source: None, tags: None, command: "CAP".to_string(), params: vec!["LS".to_string(), "302".to_string()]};
    transport.send(cap).await;

    let user = wire::RawMsg{source: None, tags: None, command: "USER".to_string(), params: vec![
        settings.get_str("nick").unwrap(),
        "0".to_string(),
        "*".to_string(),
        settings.get_str("name").unwrap(),
    ]};

    transport.send(user).await;

    let nick = wire::RawMsg{source: None, tags: None, command: "NICK".to_string(), params: vec!["MrBotMcBotFace".to_string()]};
    transport.send(nick).await;

    // TODO this should wait until after we get the CAP resp
    let cap_end = wire::RawMsg{source: None, tags: None, command: "CAP".to_string(), params: vec!["END".to_string()]};

    transport.send(cap_end).await;

    while let Some(result) = transport.next().await {
        match result {
            Ok(msg) => {
                match msg.command.as_ref() {
                    "PING" => {
                        let pong = wire::RawMsg{
                            source: None, 
                            tags: None, 
                            command: "PONG".to_string(), 
                            params: vec![]
                        };

                        transport.send(pong).await;
                    },
                    _ => continue
                }
            }
            Err(e) =>{
                println!("Got error receiving line! {}", e);
            }
        }
    }

    println!("Quit!");
}
