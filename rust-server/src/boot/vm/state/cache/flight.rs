use std::sync::{Condvar, Mutex};

pub struct Flight {
    state: Mutex<State>,
    cv: Condvar,
}
struct State {
    done: bool,
    result: Option<Result<wasmtime::Module, &'static str>>,
}
impl Flight {
    pub fn new() -> Self {
        Self {
            state: Mutex::new(State {
                done: false,
                result: None,
            }),
            cv: Condvar::new(),
        }
    }
    pub fn wait(&self) -> Result<wasmtime::Module, &'static str> {
        let mut g = self.state.lock().unwrap_or_else(|e| e.into_inner());
        while !g.done {
            g = self.cv.wait(g).unwrap_or_else(|e| e.into_inner());
        }
        g.result.clone().unwrap_or(Err("limit"))
    }
    pub fn finish(&self, r: Result<wasmtime::Module, &'static str>) {
        let mut g = self.state.lock().unwrap_or_else(|e| e.into_inner());
        g.result = Some(r);
        g.done = true;
        self.cv.notify_all();
    }
}
