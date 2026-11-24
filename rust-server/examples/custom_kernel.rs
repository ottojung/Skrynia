fn main() -> Result<(), Box<dyn std::error::Error>> {
    libskrynia::run_with(|| {
        let mut calls = libskrynia::Syscalls::new();
        calls.register("double", |_, value| Ok(value.wrapping_mul(2)))?;
        Ok(calls)
    })
}
