const ERR_OVERFLOW: i32 = -11;
const ERR_PERM: i32 = -4;

struct Vm {
    last_overflow: i32,
    execs: u32,
}

impl Vm {
    fn new() -> Self {
        Self {
            last_overflow: -1,
            execs: 0,
        }
    }
    fn eval(&mut self, out_len: usize, cap: usize) -> i32 {
        self.execs += 1;
        if out_len > cap {
            self.last_overflow = out_len as i32;
            ERR_OVERFLOW
        } else {
            self.last_overflow = -1;
            out_len as i32
        }
    }
    fn invoke(cb: i32) -> i32 {
        if cb < 0 { ERR_PERM } else { 0 }
    }
}

#[test]
fn overflow_single_shot_and_no_callback() {
    let mut vm = Vm::new();
    assert_eq!(vm.eval(9, 4), ERR_OVERFLOW);
    assert_eq!(vm.last_overflow, 9);
    assert_eq!(vm.execs, 1);
    assert_eq!(Vm::invoke(-1), ERR_PERM);
}
