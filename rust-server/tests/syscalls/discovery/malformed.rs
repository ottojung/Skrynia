#[test]
fn rejects_malformed_names_signature_shapes_and_lists() {
    for body in [
        r#"[{"name":"bad name","params":[],"results":[]}]"#,
        r#"[{"name":".","params":[],"results":[]}]"#,
        r#""double""#,
        r#"{"syscalls":["double"]}"#,
        r#"[1]"#,
        r#"{}"#,
        r#"not json"#,
        r#"[{"name":"double","params":["f32"],"results":[]}]"#,
        r#"[{"name":"double","params":[]}]"#,
        r#"[{"name":"a","params":[],"results":[]},{"name":"a","params":[],"results":[]}]"#,
    ] {
        assert!(super::rejects(body), "{body}");
    }
}
