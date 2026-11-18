/// Disk capability for durable registry state.
pub trait Disk {
    type Err;
    fn read_all(&self, path: &str) -> Result<Option<Vec<u8>>, Self::Err>;
    fn write_atomic(&self, path: &str, bytes: &[u8]) -> Result<(), Self::Err>;
    fn remove(&self, path: &str) -> Result<(), Self::Err>;
    fn list(&self, dir: &str) -> Result<Vec<String>, Self::Err>;
}

/// Global RAM capability; implementations must only touch RAM explicitly.
pub trait Ram {
    type Err;
    fn read(&self, off: u64, out: &mut [u8]) -> Result<(), Self::Err>;
    fn write(&self, off: u64, data: &[u8]) -> Result<(), Self::Err>;
    fn cas_u64(&self, off: u64, expect: u64, want: u64) -> Result<u64, Self::Err>;
}

/// Program evaluation capability used to invoke stored WASM.
pub trait Eval {
    type Err;
    fn eval(&self, wasm: &[u8], input: &[u8]) -> Result<Vec<u8>, Self::Err>;
}

/// Secure random bytes for secret ID minting.
pub trait Random {
    type Err;
    fn fill(&self, out: &mut [u8]) -> Result<(), Self::Err>;
}
