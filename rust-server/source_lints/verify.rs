#[test]
fn enforce_project_style() {
    super::check(std::path::Path::new("rust-server"));
}

#[test]
fn enforce_program_zero_style() {
    super::check(std::path::Path::new("program0"));
}
