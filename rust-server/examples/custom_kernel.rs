fn main() -> Result<(), Box<dyn std::error::Error>> {
    libskrynia::run_optimized(|| {
        Ok(libskrynia::Syscalls::optimized([
            libskrynia::EquivalentCall::new("double", "i", "i", |args| {
                Ok(vec![wasmtime::Val::I32(
                    args[0].unwrap_i32().wrapping_mul(2),
                )])
            })?,
        ])?)
    })
}
