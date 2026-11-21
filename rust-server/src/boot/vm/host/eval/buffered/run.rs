use super::prep::Args;
pub fn run(c: &mut wasmtime::Caller<'_, crate::boot::vm::state::Vm>, a: &Args) -> i32 {
    let p = match super::prep::load(c, a) {
        Ok(x) => x,
        Err(e) => return e,
    };
    let input =
        crate::fs::store::item::Input::new(Box::new(std::io::Cursor::new(p.req)), a.req_len as u64);
    let in_fd = match c
        .data_mut()
        .fds
        .insert(crate::fs::store::Item::Input(input))
    {
        Ok(f) => f,
        Err(_) => return crate::boot::vm::codes::ERR_LIMIT,
    };
    let out = crate::fs::store::Item::BufferedOutput(crate::fs::store::item::BufferedOutput::new(
        a.res_cap as usize,
    ));
    let out_fd = match c.data_mut().fds.insert(out) {
        Ok(f) => f,
        Err(_) => {
            let _ = c.data_mut().fds.remove(in_fd);
            return crate::boot::vm::codes::ERR_LIMIT;
        }
    };
    super::go::exec(c, a, in_fd, out_fd, p.cb, &p.module)
}
