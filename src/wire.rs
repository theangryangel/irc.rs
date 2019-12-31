use std::collections::HashMap;

pub struct RawMsg {
    tags: Option<HashMap<String, Option<String>>>,
    source: Option<String>,
    command: String,
    params: Vec<String>,
}

impl RawMsg {
    fn decode(x: String) -> RawMsg {
        let mut i = x.trim().chars().fuse().peekable();

        let tags: Option<HashMap<String, Option<String>>> = if i.peek() == Some(&'@') {
            let tags_string = i.by_ref().skip(1).take_while(|c| c != &' ').collect::<String>();

            Some(
                tags_string
                .split(';')
                .map(|kv| kv.split('=').collect::<Vec<&str>>())
                .map(|vec| {
                    if vec.len() == 2 {
                        (vec[0].to_string(), Some(vec[1].to_string()))
                    } else {
                        (vec[0].to_string(), None)
                    }
                })
                .collect()
            )
        } else {
            None
        };

        let source: Option<String> = if i.peek() == Some(&':') {
            Some(
                i.by_ref()
                .skip(1)
                .take_while(|c| c != &' ')
                .collect::<String>()
            )
        } else {
            None
        };

        let command: String = i.by_ref().take_while(|c| c != &' ').collect::<String>();

        let mut params: Vec<String> = i.by_ref()
            .take_while(|c| c != &':')
            .collect::<String>()
            .trim()
            .split(' ')
            .map(|vec| vec.to_string())
            .collect::<Vec<String>>();

        let (lower, _) = i.size_hint();
        if lower > 0 {
            params.push(i.collect::<String>());
        }

        RawMsg{tags: tags, source: source, command: command, params: params}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn complete_test() {
        let sample = String::from("@id=234AB;rose :dan!d@localhost PRIVMSG #chan :Hey what's up!");

        let msg = RawMsg::decode(sample);
        let tags = msg.tags.unwrap();
        let source = msg.source.unwrap();
        let command = msg.command;
        let params = msg.params;
        
        for (key, value) in &tags {
            if !value.is_none() {
                println!("key {} has value {}", key, value.as_ref().unwrap());
            } else {
                println!("key {} is set", key);
            }
        }


        for param in &params {
            println!("param is {}", param);
        }

        println!("param length is {}", params.len());

        assert_eq!(None, tags.get("rose").unwrap().as_ref());
        assert_eq!("234AB", tags.get("id").unwrap().as_ref().unwrap());
        assert_eq!("dan!d@localhost", source);
        assert_eq!("PRIVMSG", command);
        assert_eq!(2, params.len());
        assert_eq!("#chan", params[0]);
        assert_eq!("Hey what's up!", params[1]);
    }

    #[test]
    fn no_tags_test() {
        let sample = String::from(":irc.example.com CAP LS * :multi-prefix extended-join sasl");

        let msg = RawMsg::decode(sample);
        let source = msg.source.unwrap();

        assert_eq!(None, msg.tags);
        assert_eq!("irc.example.com", source);
        assert_eq!("CAP", msg.command);
        assert_eq!("LS", msg.params[0]);
        assert_eq!("*", msg.params[1]);
        assert_eq!(3, msg.params.len());
        assert_eq!("multi-prefix extended-join sasl", msg.params[2]);
    }

    #[test]
    fn no_tags_no_source_test() {
        let sample = String::from("CAP LS * :multi-prefix extended-join sasl");

        let msg = RawMsg::decode(sample);

        assert_eq!(None, msg.tags);
        assert_eq!(None, msg.source);
        assert_eq!("CAP", msg.command);
        assert_eq!("LS", msg.params[0]);
        assert_eq!("*", msg.params[1]);
        assert_eq!(3, msg.params.len());
        assert_eq!("multi-prefix extended-join sasl", msg.params[2]);
    }

    #[test]
    fn no_tags_no_trailing_test() {
        let sample = String::from(":dan!d@localhost PRIVMSG #chan Hey!");

        let msg = RawMsg::decode(sample);

        assert_eq!(None, msg.tags);
        assert_eq!("dan!d@localhost", msg.source.unwrap());
        assert_eq!("PRIVMSG", msg.command);
        assert_eq!("#chan", msg.params[0]);
        assert_eq!("Hey!", msg.params[1]);
        assert_eq!(2, msg.params.len());
    }
}
