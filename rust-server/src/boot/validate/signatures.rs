// ABI 3.1: u64 linear-memory pointers/lengths ("l"), bulk and byte I/O.
pub fn expected(name: &str) -> Option<(&'static str, &'static str)> {
    Some(match name {
        "open_disk" | "mkdir" => ("lli", "i"),
        "exists" | "is_dir" | "delete" | "open_directory" | "secure_random_bits" => ("ll", "i"),
        "rename" => ("llll", "i"),
        "next_entry" => ("ill", "i"),
        "truncate" => ("il", "i"),
        "close" | "sync" => ("i", "i"),
        "read" | "write" => ("ill", "l"),
        "read1" => ("i", "i"),
        "write1" => ("ii", "i"),
        "seek" => ("il", "l"),
        "cas1" => ("ilii", "i"),
        "open_ram" => ("", "i"),
        "now" => ("", "l"),
        "eval" => ("lliii", "l"),
        _ => return None,
    })
}
