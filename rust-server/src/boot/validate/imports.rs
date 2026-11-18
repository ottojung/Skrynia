pub fn check(module: &wasmtime::Module) -> Result<(), String> {
    for import in module.imports() {
        if import.module() != "skrynia" {
            return Err(format!("root imports forbidden module {}", import.module()));
        }
        let (args, result) = super::signatures::expected(import.name())
            .ok_or_else(|| format!("root import forbidden: {}", import.name()))?;
        match import.ty() {
            wasmtime::ExternType::Func(ty) if super::shape::matches(&ty, args, result) => {}
            _ => return Err(format!("root import has invalid type: {}", import.name())),
        }
    }
    Ok(())
}
