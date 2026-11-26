// Target ABI signatures: u64 shared-RAM pointers/lengths ("l") and byte-scalar
// I/O. Removed old-ABI inventory is not registered (target-abi.md section 5).
pub fn expected(name: &str) -> Option<(&'static str, &'static str)> {
    Some(match name {
        "open_disk" | "mkdir" => ("lli", "i"),
        "exists" | "is_dir" | "delete" | "open_directory" | "secure_random_bits" => ("ll", "i"),
        "rename" => ("llll", "i"),
        "next_entry" => ("ill", "i"),
        "truncate" => ("il", "i"),
        "close" | "sync" => ("i", "i"),
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
