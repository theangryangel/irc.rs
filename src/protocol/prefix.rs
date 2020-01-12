use std::str;

/*
 * Helper to store, parse and encode IRC prefix
 */

pub struct Prefix {
    // nick or server
    pub nick: String,
    
    pub user: Option<String>,
    pub host: Option<String>
}

impl Prefix {

    pub fn from_string(x: String) -> Prefix {
        let mut i = x.chars().fuse();

        let nick = i.by_ref().take_while(|c| c != &'!').collect::<String>();

        let (lower, _) = i.size_hint();

        let (user, host) = if lower > 0 {
            let user = i.by_ref().take_while(|c| c != &'@').collect::<String>();
            let host = i.by_ref().collect::<String>();

            (Some(user), Some(host))
        } else {
            (None, None)
        };

        Prefix{nick: nick, user: user, host: host}
    }

    pub fn to_string(&self) -> String {
        let mut s = String::new();

        s.push_str(self.nick.as_ref());

        if !self.user.is_none() {
            s.push_str(format!("!{}", self.user.as_ref().unwrap()).as_ref());
            if !self.host.is_none() {
                s.push_str(format!("@{}", self.host.as_ref().unwrap()).as_ref());
            }
        }

        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prefix_only_server_from_string_test() {
        let sample: String = "tolkien.freenode.net".to_string();
        let prefix = Prefix::from_string(sample.clone());

        assert_eq!(prefix.nick, sample);
        assert!(prefix.user.is_none());
        assert!(prefix.host.is_none());
        assert_eq!(prefix.to_string(), sample);
    }

    #[test]
    fn prefix_from_string_test() {
        let sample: String = "the_angry_angel!~karl@127.0.0.1".to_string();
        let prefix: Prefix = Prefix::from_string(sample.clone());
    
        assert_eq!(prefix.nick, "the_angry_angel");
        assert_eq!(prefix.user.as_ref().unwrap(), "~karl");
        assert_eq!(prefix.host.as_ref().unwrap(), "127.0.0.1");

        assert_eq!(prefix.to_string(), sample);
    }
}
