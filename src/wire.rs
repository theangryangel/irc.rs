use std::collections::BTreeMap;
use std::str;

pub struct RawMsg {
    tags: Option<BTreeMap<String, Option<String>>>,
    source: Option<String>,
    command: String,
    params: Vec<String>,
}

impl RawMsg {
    fn from_string(x: String) -> RawMsg {
        let mut i = x.trim().chars().fuse().peekable();

        let tags: Option<BTreeMap<String, Option<String>>> = if i.peek() == Some(&'@') {
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

    fn to_string(&self) -> String {
        let mut s = String::new();

        if !self.tags.is_none() {
            let tags = self.tags.as_ref().unwrap().iter().map(|kv|
                if kv.1.is_none() {
                    format!("{}", kv.0)
                } else {
                    format!("{}={}", kv.0, kv.1.as_ref().unwrap())
                }
            )
            .collect::<Vec<String>>();

            if tags.len() > 0 {
                s.push_str(format!("@{} ", tags.join(";")).as_ref());
            }
        }
        

        if !self.source.is_none() {
            s.push_str(format!(":{} ", self.source.as_ref().unwrap()).as_ref());
        }

        s.push_str(&self.command);
        
        for param in &self.params {
            if param.contains(" ") {
                s.push_str(" :");
            } else {
                s.push_str(" ");
            }

            s.push_str(&param);
        }

        s.push_str("\r\n");
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_string_complete_test() {
        let sample = String::from("@id=234AB;rose :dan!d@localhost PRIVMSG #chan :Hey what's up!\r\n");

        let msg = RawMsg::from_string(sample);
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
    fn from_string_no_tags_test() {
        let sample = String::from(":irc.example.com CAP LS * :multi-prefix extended-join sasl\r\n");

        let msg = RawMsg::from_string(sample);
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
    fn from_string_no_tags_no_source_test() {
        let sample = String::from("CAP LS * :multi-prefix extended-join sasl\r\n");

        let msg = RawMsg::from_string(sample);

        assert_eq!(None, msg.tags);
        assert_eq!(None, msg.source);
        assert_eq!("CAP", msg.command);
        assert_eq!("LS", msg.params[0]);
        assert_eq!("*", msg.params[1]);
        assert_eq!(3, msg.params.len());
        assert_eq!("multi-prefix extended-join sasl", msg.params[2]);
    }

    #[test]
    fn from_string_no_tags_no_trailing_test() {
        let sample = String::from(":dan!d@localhost PRIVMSG #chan Hey!\r\n");

        let msg = RawMsg::from_string(sample);

        assert_eq!(None, msg.tags);
        assert_eq!("dan!d@localhost", msg.source.unwrap());
        assert_eq!("PRIVMSG", msg.command);
        assert_eq!("#chan", msg.params[0]);
        assert_eq!("Hey!", msg.params[1]);
        assert_eq!(2, msg.params.len());
    }

    #[test]
    fn to_string_simple_test() {
        let sample = RawMsg{
            tags: None, 
            source: None, 
            command: "PRIVMSG".to_string(), 
            params: vec![
                "#chan".to_string(), 
                "Hello world!".to_string()
            ]
        };

        assert_eq!("PRIVMSG #chan :Hello world!\r\n", sample.to_string());
    }

    #[test]
    fn to_string_source_test() {
        let sample = RawMsg{
            tags: None, 
            source: Some("dan!d@localhost".to_string()), 
            command: "PRIVMSG".to_string(), 
            params: vec![
                "#chan".to_string(), 
                "Hello world!".to_string()
            ]
        };

        assert_eq!(":dan!d@localhost PRIVMSG #chan :Hello world!\r\n", sample.to_string());
    }

    #[test]
    fn to_string_complete_test() {
        let tags: BTreeMap<String, Option<String>> = vec![
            ("id".to_string(), Some("234AB".to_string())),
            ("rose".to_string(), None),
        ].into_iter().collect();

        let sample = RawMsg{
            tags: Some(tags), 
            source: Some("dan!d@localhost".to_string()), 
            command: "PRIVMSG".to_string(), 
            params: vec![
                "#chan".to_string(), 
                "Hello world!".to_string()
            ]
        };

        assert_eq!("@id=234AB;rose :dan!d@localhost PRIVMSG #chan :Hello world!\r\n", sample.to_string());
    }

}
