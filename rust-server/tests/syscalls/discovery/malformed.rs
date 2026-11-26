#[test]
fn rejects_malformed_names_signature_shapes_and_lists() {
    for body in [
        r#"{"functions":[{"name":"bad name","params":[],"results":[]}]}"#,
        r#"{"functions":[{"name":".","params":[],"results":[]}]}"#,
        r#""double""#,
        r#"{"syscalls":["double"]}"#,
        r#"{"functions":[1]}"#,
        r#"{}"#,
        r#"not json"#,
        r#"{"functions":[{"name":"double","params":["f32"],"results":[]}]}"#,
        r#"{"functions":[{"name":"double","params":[]}]}"#,
        r#"{"functions":[{"name":"a","params":[],"results":[]},{"name":"a","params":[],"results":[]}]}"#,
    ] {
        assert!(super::rejects(body), "{body}");
    }
}
