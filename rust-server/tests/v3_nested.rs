const ERR_LIMIT: i32 = -10;

struct Vm {
    depth: i32,
    execs: u32,
}

impl Vm {
    fn new() -> Self {
        Self { depth: 0, execs: 0 }
    }
    fn eval(&mut self, nested: bool) -> i32 {
        if self.depth >= 32 {
            return ERR_LIMIT;
        }
        self.depth += 1;
        self.execs += 1;
        let r = if nested { self.callback_eval() } else { 0 };
        self.depth -= 1;
        r
    }
    fn callback_eval(&mut self) -> i32 {
        self.eval(false)
    }
}

#[test]
fn callback_may_perform_nested_eval() {
    let mut vm = Vm::new();
    assert_eq!(vm.eval(true), 0);
    assert!(vm.execs >= 2);
}
