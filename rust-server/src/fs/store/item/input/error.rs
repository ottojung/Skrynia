pub(super) fn read_error(e: std::io::Error) -> &'static str {
    if e.kind() == std::io::ErrorKind::InvalidData {
        "limit"
    } else {
        "io"
    }
}
