use super::Value;
mod check;
mod codec;
mod merge;
mod read;
mod seek;
mod write;
pub use read::Reader;
use std::path::PathBuf;
pub use write::Writer;
#[derive(Clone)]
pub struct Run {
    pub path: PathBuf,
    pub len: u64,
}

mod bound;
