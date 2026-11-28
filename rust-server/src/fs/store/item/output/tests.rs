use super::Output;
use std::io::{self, Write};

struct RejectEmpty;
impl Write for RejectEmpty {
    fn write(&mut self, src: &[u8]) -> io::Result<usize> {
        assert!(!src.is_empty(), "zero-byte write reached transport");
        Ok(src.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[test]
fn zero_length_write_does_not_touch_native_sink() {
    let mut output = Output::new(Box::new(RejectEmpty));
    assert_eq!(output.write(&[]), Ok(0));
    assert_eq!(output.position(), 0);
    assert_eq!(output.write(&[7]), Ok(1));
    assert_eq!(output.position(), 1);
}
