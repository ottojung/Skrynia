use skrynia_server::{boot, conf, http};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = conf::Config::from_env()?;
    let kernel = boot::Kernel::start(&cfg.root_wasm, &cfg.data_dir)?;
    let addr = std::env::var("SKRYNIA_BIND").unwrap_or_else(|_| "127.0.0.1:8080".into());
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, http::router(kernel)).await?;
    Ok(())
}
