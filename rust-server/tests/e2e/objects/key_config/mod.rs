mod invalid;
mod valid;
fn data_dir() -> std::path::PathBuf {
    std::env::temp_dir().join(format!("skrynia-keyconfig-{}", rand::random::<u64>()))
}
