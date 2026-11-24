use libskrynia::conf;
use std::ffi::OsString;

#[test]
fn config_requires_separate_root_dirs() {
    assert!(
        conf::Config::from_lookup(|k| match k {
            "SKRYNIA_ROOT_WASM" => Some(OsString::from("r.wasm")),
            "SKRYNIA_DATA_DIR" => Some(OsString::from("/legacy")),
            _ => None,
        })
        .is_err()
    );
    let c = conf::Config::from_lookup(|k| match k {
        "SKRYNIA_ROOT_WASM" => Some(OsString::from("r.wasm")),
        "SKRYNIA_OPERATOR_DIR" => Some(OsString::from("/op")),
        "SKRYNIA_USER_DIR" => Some(OsString::from("/us")),
        _ => None,
    })
    .unwrap();
    assert_ne!(c.operator_dir, c.user_dir);
}
