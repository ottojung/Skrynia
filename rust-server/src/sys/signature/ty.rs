pub fn valid(types: &str) -> bool {
    types.bytes().all(|b| b == b'i' || b == b'l')
}

pub fn values(types: &str) -> impl Iterator<Item = wasmtime::ValType> + '_ {
    types.bytes().map(|b| {
        if b == b'i' {
            wasmtime::ValType::I32
        } else {
            wasmtime::ValType::I64
        }
    })
}

pub fn letter(ty: wasmtime::ValType) -> char {
    match ty {
        wasmtime::ValType::I32 => 'i',
        wasmtime::ValType::I64 => 'l',
        _ => '?',
    }
}
