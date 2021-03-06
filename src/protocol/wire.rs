use crate::protocol::tags::{Tags, TagValue};
use crate::protocol::prefix::Prefix;

pub struct RawMsg {
    pub tags: Option<Tags>,
    pub source: Option<Prefix>,
    pub command: String,
    pub params: Vec<String>,
}

impl RawMsg {
    pub fn new(command: String, optional_params: Option<Vec<String>>) -> RawMsg {
        let params = if optional_params.is_none() {
            vec![]
        } else {
            optional_params.unwrap()
        };

        RawMsg {
            tags: None,
            source: None,
            command: command,
            params: params
        }
    }

    pub fn from_string(x: String) -> RawMsg {
        let mut i = x.chars().fuse().peekable();

        let tags: Option<Tags> = if i.peek() == Some(&'@') {
            let tags_string = i.by_ref().skip(1).take_while(|c| c != &' ').collect::<String>();

            Some(
                Tags::from_string(tags_string)
            )
        } else {
            None
        };

        let source: Option<Prefix> = if i.peek() == Some(&':') {
            let prefix = i.by_ref()
                .skip(1)
                .take_while(|c| c != &' ')
                .collect::<String>();
            Some(
                Prefix::from_string(prefix)
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

    pub fn to_string(&self) -> String {
        let mut s = String::new();

        if !self.tags.is_none() && !self.tags.is_none() {
            s.push_str(format!("@{} ", self.tags.as_ref().unwrap().to_string().unwrap()).as_ref());
        }
        
        if !self.source.is_none() {
            s.push_str(format!(":{} ", self.source.as_ref().unwrap().to_string()).as_ref());
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

        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_string_complete_test() {
        let sample = String::from("@id=234AB;rose :dan!d@localhost PRIVMSG #chan :Hey what's up!");

        let msg = RawMsg::from_string(sample);
        let tags = msg.tags.unwrap();
        let source = msg.source.unwrap();
        let command = msg.command;
        let params = msg.params;

        assert!(match tags.get("rose".to_string()).unwrap() {
            TagValue::True => true,
            _ => false
        });

        assert!(match tags.get("id".to_string()).unwrap() {
            TagValue::String(s) => s == "234AB",
            _ => false
        });
        assert_eq!("dan!d@localhost", source.to_string());
        assert_eq!("PRIVMSG", command);
        assert_eq!(2, params.len());
        assert_eq!("#chan", params[0]);
        assert_eq!("Hey what's up!", params[1]);
    }

    #[test]
    fn from_string_no_tags_test() {
        let sample = String::from(":irc.example.com CAP LS * :multi-prefix extended-join sasl");

        let msg = RawMsg::from_string(sample);
        let source = msg.source.unwrap();

        assert!(msg.tags.is_none());
        assert_eq!("irc.example.com", source.to_string());
        assert_eq!("CAP", msg.command);
        assert_eq!("LS", msg.params[0]);
        assert_eq!("*", msg.params[1]);
        assert_eq!(3, msg.params.len());
        assert_eq!("multi-prefix extended-join sasl", msg.params[2]);
    }

    #[test]
    fn from_string_no_tags_no_source_test() {
        let sample = String::from("CAP LS * :multi-prefix extended-join sasl");

        let msg = RawMsg::from_string(sample);

        assert!(msg.tags.is_none());
        assert!(msg.source.is_none());
        assert_eq!("CAP", msg.command);
        assert_eq!("LS", msg.params[0]);
        assert_eq!("*", msg.params[1]);
        assert_eq!(3, msg.params.len());
        assert_eq!("multi-prefix extended-join sasl", msg.params[2]);
    }

    #[test]
    fn from_string_no_tags_no_trailing_test() {
        let sample = String::from(":dan!d@localhost PRIVMSG #chan Hey!");

        let msg = RawMsg::from_string(sample);

        assert!(msg.tags.is_none());
        assert_eq!("dan!d@localhost", msg.source.unwrap().to_string());
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

        assert_eq!("PRIVMSG #chan :Hello world!", sample.to_string());
    }

    #[test]
    fn to_string_source_test() {
        let sample = RawMsg{
            tags: None, 
            source: Some(Prefix{
                nick: "dan".to_string(),
                user: Some("d".to_string()),
                host: Some("localhost".to_string())
            }), 
            command: "PRIVMSG".to_string(), 
            params: vec![
                "#chan".to_string(), 
                "Hello world!".to_string()
            ]
        };

        assert_eq!(":dan!d@localhost PRIVMSG #chan :Hello world!", sample.to_string());
    }

    #[test]
    fn to_string_complete_test() {
        let sample = RawMsg{
            tags: Some(Tags::from_string("id=234AB;rose".to_string())), 
            source: Some(Prefix{
                nick: "dan".to_string(),
                user: Some("d".to_string()),
                host: Some("localhost".to_string())
            }), 
            command: "PRIVMSG".to_string(), 
            params: vec![
                "#chan".to_string(), 
                "Hello world!".to_string()
            ]
        };

        assert_eq!("@id=234AB;rose :dan!d@localhost PRIVMSG #chan :Hello world!", sample.to_string());
    }

}
