use super::imports;
use std::path::Path;

#[test]
fn qualified_visibility_import_cannot_descend_into_grandchild() {
    let root = std::env::temp_dir().join(format!("ivanka-deep-{}", std::process::id()));
    let folder = root.join("src").join("alpha");
    std::fs::create_dir_all(folder.join("beta")).unwrap();
    std::fs::write(folder.join("beta").join("gamma.rs"), "").unwrap();
    let mut issues = Vec::new();
    imports::check(
        &folder.join("main.rs"),
        "pub(super) use super::beta::gamma::Item;",
        &mut issues,
    );
    assert_eq!(issues.len(), 1);
    std::fs::remove_dir_all(root).unwrap();
}
#[test]
fn module_source_cannot_point_into_descendant_directory() {
    let mut issues = Vec::new();
    imports::check(
        Path::new("src/main.rs"),
        "#[path = \"inner/nested.rs\"] mod nested;",
        &mut issues,
    );
    assert_eq!(issues.len(), 1);
}
