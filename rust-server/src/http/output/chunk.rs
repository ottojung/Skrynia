use axum::body::Bytes;

const fn byte_values() -> [u8; 256] {
    let mut values = [0; 256];
    let mut i = 0;
    while i < values.len() {
        values[i] = i as u8;
        i += 1;
    }
    values
}
static BYTE_VALUES: [u8; 256] = byte_values();

pub fn bytes(src: &[u8]) -> Bytes {
    match src {
        [byte] => {
            let i = *byte as usize;
            Bytes::from_static(&BYTE_VALUES[i..i + 1])
        }
        _ => Bytes::copy_from_slice(src),
    }
}
