// Target K1 signatures use u64 shared-RAM pointers/lengths ("l") and
// byte-scalar I/O. Legacy entries remain until K2 ports eval/tests.
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
        "open_ram" | "try_mutation_lock" | "release_mutation_lock" => ("", "i"),
        "now" => ("", "l"),
        // Legacy old-ABI imports, pending removal by K0/K2.
        "read" | "write" => ("iii", "i"),
        "cas_ram_u64" => ("illli", "i"),
        "fs_delegate_scope" => ("iiii", "i"),
        "eval" | "eval_delegate" => ("iiiii", "i"),
        "eval_buffered" => ("iiiiiii", "i"),
        _ => return None,
    })
}
