mod child;
mod fds;
mod io;
mod probe;
mod prog;
mod run;
mod slots;
mod worker;

pub mod attest {
    use sha2::{Digest, Sha256};
    const MAGIC: &[u8; 8] = b"SKYRNM01";
    const HEAD: usize = 8 + 32;
    pub fn wrap(payload: &[u8]) -> Vec<u8> {
        let mut out = Vec::with_capacity(HEAD + payload.len());
        out.extend_from_slice(MAGIC);
        out.extend_from_slice(&Sha256::digest(payload));
        out.extend_from_slice(payload);
        out
    }
    pub fn verify(envelope: &[u8]) -> Result<&[u8], &'static str> {
        if envelope.len() < HEAD + 1 || !envelope.starts_with(MAGIC) {
            return Err("invalid");
        }
        let digest = &envelope[8..HEAD];
        let payload = &envelope[HEAD..];
        if Sha256::digest(payload).as_slice() != digest {
            return Err("invalid");
        }
        Ok(payload)
    }
}

pub fn bounded(engine: &wasmtime::Engine, code: &[u8]) -> Result<wasmtime::Module, &'static str> {
    run::bounded(engine, code)
}

pub fn engine() -> Result<wasmtime::Engine, String> {
    wasmtime::Engine::new(&crate::boot::limits::engine_config()?).map_err(|e| format!("{e}"))
}

pub fn worker_main() -> ! {
    worker::main()
}
