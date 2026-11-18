use super::codes::*;
pub fn map(e: &'static str) -> i32 {
    match e {
        "badfd" => ERR_BADFD,
        "notfound" | "missing" => ERR_NOT_FOUND,
        "exists" => ERR_EXISTS,
        "perm" | "delete" | "mkdir" | "rename" | "stat" | "mode" => ERR_PERM,
        "io" | "open" => ERR_IO,
        "nospace" => ERR_NOSPACE,
        "range" | "oob" | "unaligned" => ERR_RANGE,
        "overflow" => ERR_OVERFLOW,
        "unsupported" => ERR_UNSUPPORTED,
        "invalid" | "escape" | "null" | "empty" => ERR_INVALID,
        "limit" => ERR_LIMIT,
        _ => ERR_FAULT,
    }
}
pub fn trap(e: wasmtime::Error) -> i32 {
    let s = e.to_string().to_ascii_lowercase();
    if s.contains("all fuel consumed") || s.contains("out of fuel") {
        return super::codes::ERR_LIMIT;
    }
    super::codes::ERR_FAULT
}
