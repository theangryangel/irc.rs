use std::collections::BTreeMap;
use std::str;
use std::iter::Iterator;

/*
 * Helper to store, parse and encode IRCv3 tags
 */

pub enum TagValue {
    String(String),
    True
}

pub struct Tags {
    collection: BTreeMap<String, TagValue>,
}

impl Tags {

    pub fn from_string(x: String) -> Tags {
        let collection: BTreeMap<String, TagValue> = x.split(';')
            .map(|kv| 
                kv.split('=').collect::<Vec<&str>>()
            )
            .map(|vec| {
                if vec.len() == 2 {
                    (vec[0].to_string(), TagValue::String(vec[1].to_string()))
                } else {
                    (vec[0].to_string(), TagValue::True)
                }
            })
            .collect();

        Tags{collection: collection}
    }

    pub fn to_string(&self) -> Option<String> {
        if self.collection.len() <= 0 {
            return None
        }

        Some(
            self.collection.iter().map(|(k, v)|
                match v {
                    TagValue::String(s) => format!("{}={}", k, s),
                    TagValue::True => format!("{}", k)
                }
            )
            .collect::<Vec<String>>()
            .join(";")
            .to_string()
        )
    }

    pub fn get(&self, key: String) -> Option<&TagValue> {
        return self.collection.get(&key);
    }

    pub fn iter(&self) -> std::collections::btree_map::Iter<'_, String, TagValue> {
        return self.collection.iter();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_string_test() {
        let sample: String = "id=123123;rose".to_string();
        let tags: Tags = Tags::from_string(sample.clone());
    
        /*
        for (key, value) in tags.iter() {
            match value {
                TagValue::String(s) => println!("tag - {}={}", key, s),
                TagValue::True => println!("tag - {}=true", key)
            }
        }
        */

        assert!(match tags.get("rose".to_string()).unwrap() {
            TagValue::True => true,
            _ => false
        });

        assert!(match tags.get("id".to_string()).unwrap() {
            TagValue::String(s) => s == "123123",
            _ => false
        });

        assert!(tags.to_string().unwrap() == sample);
    }
}
