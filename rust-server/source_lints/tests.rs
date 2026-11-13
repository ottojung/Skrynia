use super::{imports, lines};
use std::path::Path;

#[test]
fn ignores_comments_and_blank_lines() {
    assert_eq!(lines::code_lines("// ignore\n\nlet x=1; // comment\n"), 1);
    assert_eq!(lines::code_lines("/* first\nsecond */\nfn f() {}\n"), 1);
}
#[test]
fn counts_real_code_lines() {
    assert_eq!(lines::code_lines("fn main() {\n  let a = \"//\";\n}\n"), 3);
}
#[test]
fn rejects_non_sibling_imports() {
    let root = std::env::temp_dir().join(format!("skrynia-lint-{}", std::process::id()));
    let sub = root.join("src").join("alpha");
    std::fs::create_dir_all(sub.join("beta")).unwrap();
    std::fs::write(sub.join("beta").join("mod.rs"), "").unwrap();
    let file = sub.join("main.rs");
    let mut issues = Vec::new();
    imports::check(&file, "use super::beta::modname;\n", &mut issues);
    assert!(issues.is_empty());
    imports::check(&file, "use super::super::beta;\n", &mut issues);
    assert_eq!(issues.len(), 1);
    std::fs::remove_dir_all(root).unwrap();
}
#[test]
fn normal_sibling_imports_are_allowed() {
    let mut issues = Vec::new();
    imports::check(
        Path::new("src/assembler/run.rs"),
        "use super::reads::{K, Reads};\nuse std::io;\n",
        &mut issues,
    );
    assert!(issues.is_empty());
}
