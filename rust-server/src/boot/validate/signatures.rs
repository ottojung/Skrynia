pub fn expected(name: &str) -> Option<(&'static str, &'static str)> {
    Some(match name {
        "open_disk" | "read" | "write" | "mkdir" | "next_entry" => ("iii", "i"),
        "truncate" | "seek" => ("il", "i"),
        "close" | "sync" => ("i", "i"),
        "exists" | "delete" | "is_dir" | "open_directory" | "secure_random_bits" => ("ii", "i"),
        "rename" => ("iiii", "i"),
        "open_ram" | "try_mutation_lock" | "release_mutation_lock" => ("", "i"),
        "cas_ram_u64" => ("illli", "i"),
        "now" => ("", "l"),
        "eval" | "eval_delegate" => ("iiiii", "i"),
        _ => return None,
    })
}
