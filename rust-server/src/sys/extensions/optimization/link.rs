use super::Native;
use super::context::Context;
use crate::Signature;

pub(super) type Link = Box<
    dyn FnOnce(&mut wasmtime::Linker<crate::boot::vm::state::Vm>, &str) -> Result<(), String>
        + Send,
>;

pub(super) fn native(signature: Signature, handler: Native) -> Result<Link, String> {
    let expected = signature.clone();
    Ok(Box::new(move |linker, name| {
        let ty = signature.func_type(linker.engine());
        let call = handler.clone();
        linker
            .func_new("syscall", name, ty, move |_caller, args, results| {
                let values = call(args).map_err(wasmtime::Error::msg)?;
                crate::sys::signature::fill(&expected, &values, results)
                    .map_err(wasmtime::Error::msg)
            })
            .map_err(|e| e.to_string())?;
        Ok(())
    }))
}

pub(super) fn context(signature: Signature, handler: Context) -> Result<Link, String> {
    let expected = signature.clone();
    Ok(Box::new(move |linker, name| {
        let ty = signature.func_type(linker.engine());
        let call = handler.clone();
        linker
            .func_new("syscall", name, ty, move |mut caller, args, results| {
                let values = call(&mut caller, args).map_err(wasmtime::Error::msg)?;
                crate::sys::signature::fill(&expected, &values, results)
                    .map_err(wasmtime::Error::msg)
            })
            .map_err(|e| e.to_string())?;
        Ok(())
    }))
}
