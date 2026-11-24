mod shutdown;
/// Run the HTTP reference implementation.
pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    launch(|| Ok(super::extensions::Syscalls::from_env()?))
}

/// Run an equivalent implementation instead of the HTTP reference transport.
pub fn run_optimized(
    configure: impl FnOnce() -> Result<super::extensions::Syscalls, Box<dyn std::error::Error>>,
) -> Result<(), Box<dyn std::error::Error>> {
    launch(configure)
}

fn launch(
    configure: impl FnOnce() -> Result<super::extensions::Syscalls, Box<dyn std::error::Error>>,
) -> Result<(), Box<dyn std::error::Error>> {
    if std::env::args().nth(1).as_deref() == Some("--compile-worker") {
        crate::boot::vm::compile::worker_main();
    }
    let syscalls = configure()?;
    tokio::runtime::Runtime::new()?.block_on(serve(syscalls))
}

async fn serve(syscalls: super::extensions::Syscalls) -> Result<(), Box<dyn std::error::Error>> {
    let cfg = crate::conf::Config::from_env()?;
    let kernel = crate::boot::Kernel::start_with_syscalls(
        &cfg.root_wasm,
        &cfg.operator_dir,
        &cfg.user_dir,
        syscalls,
    )?;
    let addr = std::env::var("SKRYNIA_BIND").unwrap_or_else(|_| "127.0.0.1:8080".into());
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, crate::http::router(kernel))
        .with_graceful_shutdown(shutdown::shutdown())
        .await?;
    Ok(())
}
