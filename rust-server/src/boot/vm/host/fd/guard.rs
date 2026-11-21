pub fn check(vm: &crate::boot::vm::state::Vm, fd: crate::fs::Fd) -> Result<(), i32> {
    if let Some(cap) = &vm.guest
        && !cap.allows(fd)
    {
        return Err(crate::boot::vm::codes::ERR_BADFD);
    }
    scope(vm, fd)
}
pub fn scope(vm: &crate::boot::vm::state::Vm, fd: crate::fs::Fd) -> Result<(), i32> {
    if vm.fd_scope != 0 {
        match vm.fds.owner(fd) {
            Some(o) if o == vm.fd_scope => Ok(()),
            _ => Err(crate::boot::vm::codes::ERR_BADFD),
        }
    } else {
        Ok(())
    }
}
