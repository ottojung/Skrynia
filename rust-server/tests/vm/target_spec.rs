//! Guards the normative target specification; no kernel behavior is tested here.
use std::fs;
use std::path::PathBuf;

fn read(name: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../docs")
        .join(name);
    fs::read_to_string(path).unwrap()
}

#[test]
fn target_is_distinct_from_implementation_snapshot() {
    assert!(read("v3/target-abi.md").contains("> a description of the kernel"));
    assert!(read("v3/abi.md").contains("Implementation snapshot, not the next ABI"));
    assert!(read("v3/target-qualification.md").contains("historical evidence only"));
}

#[test]
fn target_inventory_and_decisions_are_present() {
    let text = read("v3/target-abi.md");
    let names = "open_ram open_disk close read1 write1 seek truncate sync mkdir exists is_dir delete rename open_directory next_entry cas1 now secure_random_bits eval";
    for name in names.split_whitespace() {
        assert!(text.contains(name), "missing fixed function {name}");
    }
    for marker in
        "[K0-CHOICE] [UNRESOLVED] unpartitioned sequentially consistent".split_whitespace()
    {
        assert!(text.contains(marker), "missing decision marker {marker}");
    }
}

#[test]
fn intent_records_are_preserved() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../docs/intent-records");
    let mut count = 0;
    for entry in fs::read_dir(dir).unwrap() {
        let text = fs::read_to_string(entry.unwrap().path()).unwrap();
        assert!(text.contains("$id-"));
        count += 1;
    }
    assert_eq!(count, 13);
}
