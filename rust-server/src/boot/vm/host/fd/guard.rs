pub fn check(vm: &crate::boot::vm::state::Vm, fd: crate::fs::Fd) -> Result<(), i32> {
    if vm.fd_scope != 0 {
        if vm.fds.owner(fd) == Some(vm.fd_scope) {
            return Ok(());
        }
        if let Some(cap) = vm.guest
            && cap.allows(fd)
            && vm.fds.get(fd).is_some()
        {
            return Ok(());
        }
        return Err(crate::boot::vm::host::ram::args::E_BADFD);
    }
    if let Some(cap) = &vm.guest
        && !cap.allows(fd)
    {
        return Err(crate::boot::vm::host::ram::args::E_BADFD);
    }
    scope(vm, fd)
}
pub fn scope(vm: &crate::boot::vm::state::Vm, fd: crate::fs::Fd) -> Result<(), i32> {
    if vm.fd_scope != 0 {
        match vm.fds.owner(fd) {
            Some(o) if o == vm.fd_scope => Ok(()),
            _ => Err(crate::boot::vm::host::ram::args::E_BADFD),
        }
    } else {
        Ok(())
    }
}
