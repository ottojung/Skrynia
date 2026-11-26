use wasmtime::FuncType;

pub struct Binding {
    pub module: String,
    pub name: String,
    pub ty: FuncType,
}

pub fn single(module: &wasmtime::Module) -> Result<Option<Binding>, i32> {
    let mut found = None;
    for import in module.imports() {
        let ty = import.ty();
        let Some(ft) = ty.func() else {
            return Err(crate::boot::vm::codes::ERR_INVALID);
        };
        if !super::func::integer_only(ft) || found.is_some() {
            return Err(crate::boot::vm::codes::ERR_INVALID);
        }
        found = Some(Binding {
            module: import.module().to_string(),
            name: import.name().to_string(),
            ty: ft.clone(),
        });
    }
    Ok(found)
}
