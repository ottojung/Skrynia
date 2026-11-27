use super::func::Selected;
use wasmtime::{Caller, Extern, Func, FuncType, Store};

type Vm = crate::boot::vm::state::Vm;

pub struct Parent<'a, 'b> {
    pub caller: &'a mut Caller<'b, Vm>,
    pub func: Option<Func>,
}

pub fn validate(binding: Option<&FuncType>, selected: Option<&Selected>) -> Result<(), i32> {
    match (binding, selected) {
        (None, None) => Ok(()),
        (Some(b), Some(s)) if super::func::same(b, &s.ty) => Ok(()),
        _ => Err(-3),
    }
}

pub fn imports(store: &mut Store<Parent<'_, '_>>, binding: Option<FuncType>) -> Vec<Extern> {
    let Some(ty) = binding else {
        return vec![];
    };
    let f = Func::new(store, ty, |mut c, args, results| {
        let parent = c.data_mut();
        parent
            .func
            .expect("validated binding")
            .call(&mut *parent.caller, args, results)
    });
    vec![f.into()]
}
