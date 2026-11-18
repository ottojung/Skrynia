use super::FsDisk;
use std::{fs, io};
pub fn list(d: &FsDisk, dir: &str) -> io::Result<Vec<String>> {
    let rd = match fs::read_dir(d.p(dir)) {
        Ok(r) => r,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(vec![]),
        Err(e) => return Err(e),
    };
    let mut out = Vec::new();
    for e in rd {
        if let Some(n) = e?.file_name().to_str() {
            out.push(n.to_string());
        }
    }
    Ok(out)
}
