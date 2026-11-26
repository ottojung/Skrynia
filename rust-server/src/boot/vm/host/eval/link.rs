use super::binding::Binding;
use super::func::Selected;
use wasmtime::{Caller, Linker, Val};

pub fn add(
    linker: &mut Linker<crate::boot::vm::state::Vm>,
    binding: Option<Binding>,
    selected: Option<Selected>,
    arg0: i32,
    arg1: i32,
) -> Result<(), i32> {
    let (binding, selected) = match (binding, selected) {
        (None, None) => return Ok(()),
        (Some(b), Some(s)) => (b, s),
        _ => return Err(crate::boot::vm::codes::ERR_INVALID),
    };
    if !super::func::same(&binding.ty, &selected.ty) {
        return Err(crate::boot::vm::codes::ERR_INVALID);
    }
    let func = selected.func;
    linker
        .func_new(
            &binding.module,
            &binding.name,
            binding.ty,
            move |mut c: Caller<'_, crate::boot::vm::state::Vm>,
                  args: &[Val],
                  results: &mut [Val]| {
                let _context = (arg0, arg1);
                func.call(&mut c, args, results)
            },
        )
        .map_err(|_| crate::boot::vm::codes::ERR_INVALID)?;
    Ok(())
}
