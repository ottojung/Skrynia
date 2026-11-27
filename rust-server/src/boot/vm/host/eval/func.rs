use wasmtime::{Caller, Func, FuncType, ValType};

pub struct Selected {
    pub func: Func,
    pub ty: FuncType,
}

pub fn select(
    c: &mut Caller<'_, crate::boot::vm::state::Vm>,
    index: i32,
) -> Result<Option<Selected>, i32> {
    if index < 0 {
        return Ok(None);
    }
    let table = c
        .get_export("__indirect_function_table")
        .and_then(|e| e.into_table())
        .ok_or(-3)?;
    let value = table.get(&mut *c, index as u64).ok_or(-3)?;
    let wasmtime::Ref::Func(Some(func)) = value else {
        return Err(-3);
    };
    let ty = func.ty(&*c);
    if !integer_only(&ty) {
        return Err(-3);
    }
    Ok(Some(Selected { func, ty }))
}

pub fn integer_only(ty: &FuncType) -> bool {
    fn integer(v: ValType) -> bool {
        matches!(v, ValType::I32 | ValType::I64)
    }
    ty.params().all(integer) && ty.results().all(integer)
}

pub fn same(a: &FuncType, b: &FuncType) -> bool {
    fn codes<'a>(types: impl Iterator<Item = ValType> + 'a) -> Vec<u8> {
        types
            .map(|v| match v {
                ValType::I32 => 1,
                ValType::I64 => 2,
                _ => 0,
            })
            .collect()
    }
    codes(a.params()) == codes(b.params()) && codes(a.results()) == codes(b.results())
}
