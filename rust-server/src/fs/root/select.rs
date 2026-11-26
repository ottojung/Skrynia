const OPERATOR: &[u8] = b"operator/";
const USER: &[u8] = b"user/";

/// Resolve the explicit root namespace token of a syscall path. The token is
/// the sole root selector: no delegation, implicit default, overlay or
/// fallback can redirect it. The returned suffix stays confined by `clean`.
pub fn split(path: &[u8]) -> Result<(crate::fs::RootKind, &[u8]), &'static str> {
    if path.starts_with(OPERATOR) {
        Ok((crate::fs::RootKind::Operator, &path[OPERATOR.len()..]))
    } else if path.starts_with(USER) {
        Ok((crate::fs::RootKind::User, &path[USER.len()..]))
    } else {
        Err("root")
    }
}
