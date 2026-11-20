pub fn check(module: &wasmtime::Module) -> Result<(), String> {
    check_with(module, super::signatures::expected, "root")
}

pub fn check_program1(module: &wasmtime::Module, scoped: bool) -> Result<(), String> {
    check_with(
        module,
        |name| super::signatures::program1(name, scoped),
        "program1",
    )
}

fn check_with(
    module: &wasmtime::Module,
    expected: impl Fn(&str) -> Option<(&'static str, &'static str)>,
    name: &str,
) -> Result<(), String> {
    for import in module.imports() {
        if import.module() != "skrynia" {
            return Err(format!(
                "{name} imports forbidden module {}",
                import.module()
            ));
        }
        let (args, result) = expected(import.name())
            .ok_or_else(|| format!("{name} import forbidden: {}", import.name()))?;
        match import.ty() {
            wasmtime::ExternType::Func(ty) if super::shape::matches(&ty, args, result) => {}
            _ => return Err(format!("{name} import has invalid type: {}", import.name())),
        }
    }
    Ok(())
}
