mod wire;
mod tags;
mod codec;
mod prefix;

use tokio::net::TcpStream;
use tokio_util::codec::Framed;
use tokio::stream::StreamExt;
use futures::{FutureExt, SinkExt};
use config::Config;

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

    let cap = wire::RawMsg::new("CAP".to_string(), Some(vec!["LS".to_string(), "302".to_string()]));
    transport.send(cap).await;

    let user = wire::RawMsg::new("USER".to_string(), Some(vec![
        settings.get_str("nick").unwrap(),
        "0".to_string(),
        "*".to_string(),
        settings.get_str("name").unwrap(),
    ]));

    transport.send(user).await;

    let nick = wire::RawMsg::new(
        "NICK".to_string(),
        Some(vec![
            settings.get_str("nick").unwrap()
        ])
    );
    transport.send(nick).await;

    // TODO this should wait until after we get the CAP resp
    let cap_end = wire::RawMsg::new("CAP".to_string(), Some(vec!["END".to_string()]));

    transport.send(cap_end).await;

    while let Some(result) = transport.next().await {
        match result {
            Ok(msg) => {
                match msg.command.as_ref() {
                    "PING" => {
                        let pong = wire::RawMsg::new(
                            "PONG".to_string(), None
                        );

                        transport.send(pong).await;
                    },
                    "PRIVMSG" => {
                    
                        if !msg.source.is_none() {

                            let mut params = msg.params.clone();
                            params[0] = msg.source.unwrap().nick;

                            let echo = wire::RawMsg::new(
                                "PRIVMSG".to_string(), 
                                Some(params),
                            );

                            transport.send(echo).await;

                        }

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
