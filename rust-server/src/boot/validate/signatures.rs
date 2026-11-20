pub fn expected(name: &str) -> Option<(&'static str, &'static str)> {
    Some(match name {
        "open_disk" | "read" | "write" | "mkdir" | "next_entry" => ("iii", "i"),
        "truncate" | "seek" => ("il", "i"),
        "close" | "sync" => ("i", "i"),
        "exists" | "delete" | "is_dir" | "open_directory" | "secure_random_bits" => ("ii", "i"),
        "rename" => ("iiii", "i"),
        "open_ram" | "eval_output_len" | "try_mutation_lock" | "release_mutation_lock" => ("", "i"),
        "cas_ram_u64" => ("illli", "i"),
        "now" => ("", "l"),
        "eval" => ("iiiiiii", "i"),
        "run_program1" => ("ii", "i"),
        _ => return None,
    })
}

pub fn program1(name: &str) -> Option<(&'static str, &'static str)> {
    Some(match name {
        "read" | "write" => ("iii", "i"),
        "close" => ("i", "i"),
        _ => return None,
    })
}
