use std::fs;
use std::path::PathBuf;

fn read(path: &str) -> String {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    fs::read_to_string(root.join(path)).unwrap()
}

#[test]
fn fixtures_and_spec_reflect_resolved_abi() {
    let child = read("docs/v3/fixtures/child_invoke_only.wat");
    assert!(child.contains("(import \"skrynia\" \"invoke\""));
    assert!(child.contains("(export \"skrynia_main\")"));
    let bad = read("docs/v3/fixtures/forbidden_wasi.wat");
    assert!(bad.contains("wasi_snapshot_preview1"));
    let data = read("docs/v3/fixtures/data_segment.wat");
    assert!(data.contains("(data"));
    assert!(data.contains("skrynia_main"));
    let parent = read("docs/v3/fixtures/parent_callback.wat");
    assert!(parent.contains("__indirect_function_table"));
    let spec = read("docs/v3-vm-spec.md");
    assert!(spec.contains("eval_output_len() -> i32"));
    assert!(spec.contains("observed_ptr: u32) -> i32"));
    assert!(spec.contains("must never overwrite"));
}
