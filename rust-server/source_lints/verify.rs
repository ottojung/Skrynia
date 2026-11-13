#[test]
fn enforce_project_style() {
    super::check(std::path::Path::new("rust-server"));
}
