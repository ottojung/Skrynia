use crate::{Result, State, internal, missing};
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::sync::Arc;

pub async fn open(state: Arc<State>, key: &str) -> Result<(File, u64, String)> {
    let path = state.dir.join("store").join(key);
    tokio::task::spawn_blocking(move || {
        let mut file = File::open(path)?;
        let size = file.metadata()?.len();
        let mut digest = Sha256::new();
        let mut block = [0; 65536];
        loop {
            let count = file.read(&mut block)?;
            if count == 0 {
                break;
            }
            digest.update(&block[..count]);
        }
        file.seek(SeekFrom::Start(0))?;
        Ok((file, size, hex::encode(digest.finalize())))
    })
    .await
    .map_err(|_| internal())?
    .map_err(|error: std::io::Error| {
        if error.kind() == std::io::ErrorKind::NotFound {
            missing("not_found")
        } else {
            internal()
        }
    })
}
