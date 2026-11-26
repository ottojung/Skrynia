mod malformed;
mod mismatch;

pub(super) fn rejects(body: &str) -> bool {
    let discovery = super::provider::Discovery {
        status: 200,
        body: body.into(),
        ..Default::default()
    };
    let provider = super::provider::start(discovery, 1);
    let failed = libskrynia::Syscalls::http(&provider.url).is_err();
    provider.join();
    failed
}
