pub const E_BADFD: i32 = -2;
pub const E_INVAL: i32 = -3;
pub const E_NOSUP: i32 = -4;
pub const E_IO: i32 = -5;
pub const E_RANGE: i32 = -6;
pub const E_BOUNDS: i32 = -7;

pub fn map_fs(e: &'static str) -> i32 {
    match e {
        "badfd" => E_BADFD,
        "perm" | "unsupported" | "delete" | "mkdir" | "rename" | "stat" | "mode" => E_NOSUP,
        "overflow" => E_RANGE,
        "invalid" | "escape" | "null" | "empty" => E_INVAL,
        "oob" | "range" | "unaligned" | "limit" => E_BOUNDS,
        "io" | "open" => E_IO,
        _ => E_IO,
    }
}
