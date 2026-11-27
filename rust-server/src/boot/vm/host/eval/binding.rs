use wasmtime::FuncType;

pub fn single(module: &wasmtime::Module) -> Result<Option<FuncType>, i32> {
    let mut found = None;
    for import in module.imports() {
        let ty = import.ty();
        let Some(ft) = ty.func() else {
            return Err(-3);
        };
        if !super::func::integer_only(ft) || found.is_some() {
            return Err(-3);
        }
        found = Some(ft.clone());
    }
    Ok(found)
}
