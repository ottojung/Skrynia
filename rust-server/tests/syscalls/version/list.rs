fn entry(name: &str, params: &[&str], results: &[&str]) -> serde_json::Value {
    serde_json::json!({"name": name, "params": params, "results": results})
}

fn advertised() -> Vec<serde_json::Value> {
    vec![
        entry("remote", &[], &[]),
        entry("double", &["i32"], &["i32"]),
        entry("unused", &["i64"], &["i32"]),
        entry("version", &[], &[]),
        entry("syscalls_list", &[], &[]),
    ]
}

pub(super) fn discovery_body() -> String {
    let list = advertised()
        .iter()
        .map(|v| v.to_string())
        .collect::<Vec<_>>()
        .join(",");
    format!("[{list}]")
}

pub(super) fn expected() -> serde_json::Value {
    serde_json::json!([
        entry("double", &["i32"], &["i32"]),
        entry("remote", &[], &[]),
        entry("syscalls_list", &[], &[]),
        entry("unused", &["i64"], &["i32"]),
        entry("version", &[], &[]),
    ])
}
