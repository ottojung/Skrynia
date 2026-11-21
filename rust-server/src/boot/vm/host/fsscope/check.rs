pub fn root_kind(v: i32) -> Option<crate::fs::RootKind> {
    match v {
        0 => Some(crate::fs::RootKind::User),
        1 => Some(crate::fs::RootKind::Operator),
        _ => None,
    }
}
pub fn allowed(
    child: &super::super::super::cap::FsCaps,
    auth: &super::super::super::cap::FsAuth,
) -> bool {
    match auth {
        super::super::super::cap::FsAuth::Full => true,
        super::super::super::cap::FsAuth::Empty => false,
        super::super::super::cap::FsAuth::Scoped(caps) => caps.iter().any(|p| {
            child.root == p.root
                && (child.ops & !p.ops) == 0
                && super::super::super::cap::path_allowed(&p.prefix, &child.prefix)
        }),
    }
}
