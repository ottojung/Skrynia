pub fn byte(value: u8) -> Vec<u8> {
    let source = format!(
        r#"(module (memory (export "memory") 1)
      (func (export "skrynia_main") (param i32 i32 i32 i32) (result i32)
      (i32.store8 (local.get 2) (i32.const {value})) (i32.const 1)))"#
    );
    wat::parse_str(source).unwrap()
}
