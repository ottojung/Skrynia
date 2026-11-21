pub fn check(vm: &crate::boot::vm::state::Vm, fd: crate::fs::Fd) -> Result<(), i32> {
    if let Some(cap) = &vm.guest
        && !cap.allows(fd)
    {
        return Err(crate::boot::vm::codes::ERR_BADFD);
    }
    Ok(())
}
