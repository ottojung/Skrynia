mod exports;
mod imports;
mod shape;
mod signatures;

pub fn check(module: &wasmtime::Module) -> Result<(), String> {
    exports::check(module)?;
    imports::check(module)?;
    Ok(())
}

pub fn check_program1(module: &wasmtime::Module) -> Result<(), String> {
    exports::check(module)?;
    imports::check_program1(module)?;
    Ok(())
}
