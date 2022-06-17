use std::collections::HashMap;

use rand::{distributions::Alphanumeric, Rng};

pub fn generate_random_string() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(12)
        .map(char::from)
        .collect()
}

pub fn merge(v: &serde_json::Value, fields: &HashMap<String, String>) -> serde_json::Value {
    match v {
        serde_json::Value::Object(m) => {
            let mut m = m.clone();
            for (k, v) in fields {
                m.insert(k.clone(), serde_json::Value::String(v.clone()));
            }
            serde_json::Value::Object(m)
        }
        v => v.clone(),
    }
}
