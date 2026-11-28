use super::Input;
use std::io::{self, Read};

struct ScratchRead(io::Result<usize>);
impl Read for ScratchRead {
    fn read(&mut self, dst: &mut [u8]) -> io::Result<usize> {
        dst.fill(42);
        std::mem::replace(&mut self.0, Ok(0))
    }
}

#[test]
fn short_native_read_only_changes_reported_prefix() {
    let mut input = Input::new(Box::new(ScratchRead(Ok(1))), 8);
    let mut dst = [9; 8];
    assert_eq!(input.read(&mut dst), Ok(1));
    assert_eq!(dst, [42, 9, 9, 9, 9, 9, 9, 9]);
    assert_eq!(input.skip(0), Ok(1));
}

#[test]
fn failed_native_read_preserves_buffer_and_cursor() {
    let mut input = Input::new(Box::new(ScratchRead(Err(io::ErrorKind::Other.into()))), 8);
    let mut dst = [9; 8];
    assert_eq!(input.read(&mut dst), Err("io"));
    assert_eq!(dst, [9; 8]);
    assert_eq!(input.skip(0), Ok(0));
}

#[test]
fn invalid_native_count_is_rejected() {
    let mut input = Input::new(Box::new(ScratchRead(Ok(99))), 8);
    let mut dst = [9; 8];
    assert_eq!(input.read(&mut dst), Err("io"));
    assert_eq!(dst, [9; 8]);
    assert_eq!(input.skip(0), Ok(0));
}
