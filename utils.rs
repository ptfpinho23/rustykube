use serde_yaml::{Deserializer, Value};
use serde::de::Deserialize;

pub fn parse_yaml(contents: &str) -> Vec<Value> {
    Deserializer::from_str(contents)
        .map(|doc| Value::deserialize(doc).expect("Failed to deserialize YAML document"))
        .collect()
}
