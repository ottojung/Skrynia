impl Disk for FsDisk {
    type Err = std::io::Error;
    fn read_all(&self, path: &str) -> Result<Option<Vec<u8>>, Self::Err> {
        match fs::read(self.p(path)) {
            Ok(b) => Ok(Some(b)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e),
        }
    }
    fn write_atomic(&self, path: &str, bytes: &[u8]) -> Result<(), Self::Err> {
        let p = self.p(path);
        if let Some(parent) = p.parent() {
            fs::create_dir_all(parent)?;
        }
        let tmp = p.with_extension("tmp");
        fs::write(&tmp, bytes)?;
        fs::rename(tmp, p)
    }
    fn remove(&self, path: &str) -> Result<(), Self::Err> {
        match fs::remove_file(self.p(path)) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(e),
        }
    }
    fn list(&self, dir: &str) -> Result<Vec<String>, Self::Err> {
        let p = self.p(dir);
        let rd = match fs::read_dir(&p) {
            Ok(r) => r,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(vec![]),
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
}
struct Rng(u8);
