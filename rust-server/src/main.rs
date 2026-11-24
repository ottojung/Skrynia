fn main() -> Result<(), Box<dyn std::error::Error>> {
    libskrynia::run(libskrynia::Syscalls::new())
}
