use skrynia_server::{boot, conf, http};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = conf::Config::from_env()?;
    let p1 = cfg.program1_wasm.as_deref();
    let kernel = boot::Kernel::start_with(&cfg.root_wasm, &cfg.data_dir, p1)?;
    let addr = std::env::var("SKRYNIA_BIND").unwrap_or_else(|_| "127.0.0.1:8080".into());
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, http::router(kernel))
        .with_graceful_shutdown(shutdown())
        .await?;
    Ok(())
}

async fn shutdown() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{SignalKind, signal};
        let mut term = signal(SignalKind::terminate()).expect("install SIGTERM handler");
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {},
            _ = term.recv() => {},
        }
    }
    #[cfg(not(unix))]
    {
        let _ = tokio::signal::ctrl_c().await;
    }
}
