// ABI signature notation: i = i32, l = i64; all other types are forbidden.
pub fn matches(ty: &wasmtime::FuncType, params: &str, results: &str) -> bool {
    fn letter(v: wasmtime::ValType) -> char {
        match v {
            wasmtime::ValType::I32 => 'i',
            wasmtime::ValType::I64 => 'l',
            _ => '?',
        }
    }
    let actual_params: String = ty.params().map(letter).collect();
    let actual_results: String = ty.results().map(letter).collect();
    actual_params == params && actual_results == results
}
