mod malformed;
mod native;
mod small;
mod transport;

pub(super) use super::{Discovery, kernel, start};

pub(super) const N: usize = 300;

/// Extended-count frame of little-endian i32 values (`0xff` + u32 count).
pub(super) fn frame(values: &[u64]) -> Vec<u8> {
    let mut out = vec![1, 0xff];
    out.extend_from_slice(&(values.len() as u32).to_le_bytes());
    for value in values {
        out.push(0x32);
        out.extend_from_slice(&(*value as u32).to_le_bytes());
    }
    out
}

pub(super) fn types(count: usize) -> String {
    std::iter::repeat_n("\"i32\"", count)
        .collect::<Vec<_>>()
        .join(",")
}

pub(super) fn params(count: usize) -> String {
    format!("(param {})", "i32 ".repeat(count))
}

pub(super) fn results(count: usize) -> String {
    format!("(result {})", "i32 ".repeat(count))
}

pub(super) fn args(count: usize) -> String {
    (0..count)
        .map(|i| format!("(i32.const {i})"))
        .collect::<Vec<_>>()
        .join(" ")
}

pub(super) fn stores(count: usize) -> String {
    (0..count)
        .rev()
        .map(|i| format!("(call $put (i32.const {}))", i * 4))
        .collect::<Vec<_>>()
        .join(" ")
}

pub(super) fn put() -> String {
    "(func $put (param $v i32) (param $off i32) (i32.store (local.get $off) (local.get $v)))".into()
}

pub(super) fn bytes(values: &[u64]) -> Vec<u8> {
    values
        .iter()
        .flat_map(|v| (*v as u32).to_le_bytes())
        .collect()
}
