use crate::Signature;
use std::collections::BTreeMap;

pub fn to_list(signatures: &BTreeMap<String, Signature>) -> serde_json::Value {
    let list = signatures
        .iter()
        .map(|(name, sig)| {
            serde_json::json!({
                "name": name,
                "params": names(sig.params()),
                "results": names(sig.results()),
            })
        })
        .collect();
    serde_json::Value::Array(list)
}

pub fn to_discovery(signatures: &BTreeMap<String, Signature>) -> serde_json::Value {
    serde_json::json!({"functions": to_list(signatures)})
}

fn names(types: &str) -> Vec<&'static str> {
    types
        .bytes()
        .map(|b| if b == b'i' { "i32" } else { "i64" })
        .collect()
}
