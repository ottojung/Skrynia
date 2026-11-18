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
        _ => ERR_FAULT,
    }
}
