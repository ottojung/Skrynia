/// Count lines containing executable Rust, excluding blank and comment-only
/// lines. A line with code and an inline comment still counts as code.
pub fn code_lines(source: &str) -> usize {
    let mut in_block = false;
    let mut count = 0;
    for line in source.lines() {
        let bytes = line.as_bytes();
        let mut i = 0;
        let mut code = false;
        let mut in_string = false;
        let mut escaped = false;
        while i < bytes.len() {
            let pair = bytes.get(i..i + 2).unwrap_or_default();
            if in_block {
                if pair == b"*/" {
                    in_block = false;
                    i += 2;
                    continue;
                }
            } else if in_string {
                if bytes[i] == b'"' && !escaped {
                    in_string = false;
                }
                if bytes[i] == b'\\' {
                    escaped = !escaped;
                } else {
                    escaped = false;
                }
                code = true;
            } else if pair == b"//" {
                break;
            } else if pair == b"/*" {
                in_block = true;
                i += 2;
                continue;
            } else if bytes[i] == b'"' {
                in_string = true;
                code = true;
            } else if !bytes[i].is_ascii_whitespace() {
                code = true;
            }
            i += 1;
        }
        if code {
            count += 1;
        }
    }
    count
}
