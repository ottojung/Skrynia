const ERR_OVERFLOW: i32 = -11;
const ERR_PERM: i32 = -4;

struct Frame {
    last_overflow: i32,
    execs: u32,
}

impl Frame {
    fn new() -> Self {
        Self {
            last_overflow: -1,
            execs: 0,
        }
    }
    fn eval(&mut self, out_len: usize, cap: usize, cb: i32) -> i32 {
        self.execs += 1;
        if cb < -1 {
            return -1;
        }
        if out_len > cap {
            self.last_overflow = out_len as i32;
            return ERR_OVERFLOW;
        }
        self.last_overflow = -1;
        out_len as i32
    }
    fn invoke(cb: i32) -> i32 {
        if cb < 0 { ERR_PERM } else { 0 }
    }
}

#[test]
fn overflow_is_single_shot_and_length_is_recorded() {
    let mut f = Frame::new();
    assert_eq!(f.eval(9, 4, 0), ERR_OVERFLOW);
    assert_eq!(f.last_overflow, 9);
    assert_eq!(f.execs, 1);
    assert_eq!(Frame::invoke(-1), ERR_PERM);
}
