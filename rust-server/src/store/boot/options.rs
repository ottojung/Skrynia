mod length;
mod public;
use std::io;
use std::path::PathBuf;
pub struct Config {
    pub dir: PathBuf,
    pub base: String,
    pub public: String,
    pub max_object: usize,
    pub max_key_length: usize,
    pub max_buffered: usize,
}
pub fn read() -> io::Result<Config> {
    let dir = std::env::var("SKRYNIA_DATA_DIR")
        .unwrap_or_else(|_| "/var/lib/skrynia".into())
        .into();
    let public =
        std::env::var("SKRYNIA_URL").map_err(|_| io::Error::other("SKRYNIA_URL required"))?;
    let base = public::base(&public)?;
    let max_key_length = length::key()?;
    let max_object = std::env::var("SKRYNIA_MAX_OBJECT_SIZE")
        .ok()
        .map_or(Ok(10485760usize), |v| v.parse())
        .map_err(|_| io::Error::other("invalid object size"))?;
    let max_buffered = std::env::var("SKRYNIA_MAX_BUFFERED_UPLOAD_BYTES")
        .ok()
        .map_or(Ok(41943040usize), |value| value.parse())
        .map_err(|_| io::Error::other("invalid buffered upload limit"))?;
    if max_buffered == 0 || max_buffered > tokio::sync::Semaphore::MAX_PERMITS {
        return Err(io::Error::other("invalid buffered upload limit"));
    }
    if max_object == 0 {
        return Err(io::Error::other("invalid object size"));
    }
    Ok(Config {
        public,
        dir,
        base,
        max_object,
        max_key_length,
        max_buffered,
    })
}
