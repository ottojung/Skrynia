use std::sync::Arc;

pub(super) fn kernel() -> Arc<crate::boot::Kernel> {
    let dir = std::env::temp_dir().join(format!(
        "pr136-preflight-{}-{}",
        std::process::id(),
        rand::random::<u64>()
    ));
    let op = dir.join("operator");
    let usr = dir.join("user");
    std::fs::create_dir_all(&op).unwrap();
    std::fs::create_dir_all(&usr).unwrap();
    let wasm = dir.join("root.wasm");
    std::fs::write(
        &wasm,
        wat::parse_str(include_str!("../../../test-fixtures/root_stub.wat")).unwrap(),
    )
    .unwrap();
    Arc::new(crate::boot::Kernel::start(&wasm, &op, &usr).unwrap())
}
