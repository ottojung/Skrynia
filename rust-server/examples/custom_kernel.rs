fn main() -> Result<(), Box<dyn std::error::Error>> {
    libskrynia::run_optimized(|| {
        Ok(libskrynia::Syscalls::optimized([
            libskrynia::EquivalentCall::new("double", |value| Ok(value.wrapping_mul(2)))?,
        ])?)
    })
}
