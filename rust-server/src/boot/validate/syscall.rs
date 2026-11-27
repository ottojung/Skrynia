use std::collections::BTreeMap;

pub fn check(
    module: &wasmtime::Module,
    signatures: &BTreeMap<String, crate::Signature>,
) -> Result<(), String> {
    for import in module.imports() {
        if import.module() != "syscall" {
            continue;
        }
        let name = import.name();
        let ty = match import.ty() {
            wasmtime::ExternType::Func(ty) => ty,
            _ => return Err(format!("syscall import is not a function: {name}")),
        };
        let signature = signatures
            .get(name)
            .ok_or_else(|| format!("unsupported syscall: {name}"))?;
        if !signature.matches(&ty) {
            return Err(format!("syscall import has invalid type: {name}"));
        }
    }
    Ok(())
}
