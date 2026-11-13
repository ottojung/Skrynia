use std::fs;
use std::os::unix::fs::{PermissionsExt, symlink};
use std::process::Command;
#[test]
fn startup_rejects_directory_symlinks_without_mutating_targets() {
    for part in ["store", "state/grants", "state/push"] {
        let root = std::env::temp_dir().join(format!("skrynia-unsafe-{}", rand::random::<u64>()));
        let external = root.join("external");
        fs::create_dir_all(&external).unwrap();
        let sentinel = external.join("sentinel");
        fs::write(&sentinel, b"unchanged").unwrap();
        fs::set_permissions(&external, fs::Permissions::from_mode(0o700)).unwrap();
        let data = root.join("data");
        fs::create_dir_all(&data).unwrap();
        let link = data.join(part);
        fs::create_dir_all(link.parent().unwrap()).unwrap();
        symlink(&external, &link).unwrap();
        let outcome = Command::new(env!("CARGO_BIN_EXE_skrynia-server"))
            .env("SKRYNIA_DATA_DIR", &data)
            .env("SKRYNIA_URL", "http://127.0.0.1:17380")
            .output()
            .unwrap();
        assert!(!outcome.status.success(), "accepted symlinked {part}");
        assert_eq!(fs::read(&sentinel).unwrap(), b"unchanged");
        assert_eq!(
            fs::metadata(&external).unwrap().permissions().mode() & 0o777,
            0o700
        );
        fs::remove_dir_all(root).unwrap();
    }
}
