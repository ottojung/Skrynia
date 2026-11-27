pub const E_BADFD: i32 = -2;
pub const E_INVAL: i32 = -3;
pub const E_NOSUP: i32 = -4;
pub const E_IO: i32 = -5;
pub const E_RANGE: i32 = -6;
pub const E_BOUNDS: i32 = -7;
// Explicit absence code: only genuine ENOENT maps here. Never EIO/badfd.
pub const E_NOENT: i32 = -8;

pub fn map_fs(e: &'static str) -> i32 {
    match e {
        "badfd" => E_BADFD,
        "missing" => E_NOENT,
        "perm" | "unsupported" | "delete" | "mkdir" | "rename" | "stat" | "mode" => E_NOSUP,
        "overflow" => E_RANGE,
        "invalid" | "escape" | "null" | "empty" | "absolute" => E_INVAL,
        "oob" | "range" | "unaligned" | "limit" => E_BOUNDS,
        "io" | "open" => E_IO,
        _ => E_IO,
    }
}
