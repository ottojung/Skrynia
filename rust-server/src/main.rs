mod error;
mod exports;
mod grants;
mod http;
mod net;
mod operator;
mod push;
mod push_mode;
mod store;

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    if let Some(action) = args.next() {
        if args.next().is_some() {
            return Err("usage: skrynia-server [root-credentials|healthcheck]".into());
        }
        match action.as_str() {
            "root-credentials" => operator::run()?,
            "healthcheck" => net::healthcheck()?,
            _ => return Err("unknown command".into()),
        }
        return Ok(());
    }
    let server = store::State::open()?;
    let address = std::env::var("SKRYNIA_HOST").unwrap_or_else(|_| "127.0.0.1".into());
    let port = std::env::var("SKRYNIA_PORT").unwrap_or_else(|_| "17380".into());
    let listener = tokio::net::TcpListener::bind(format!("{address}:{port}")).await?;
    let router = http::router(server, push_mode::delivery_enabled());
    let options = net::NetworkOptions::load()?;
    let listener = net::BoundedListener::new(listener, options);
    axum::serve(listener, router)
        .with_graceful_shutdown(net::shutdown_signal())
        .await?;
    Ok(())
}

pub use exports::*;
