//! Starts the to-do web server on <http://127.0.0.1:3000>.
//!
//! Set the `PORT` environment variable to use a different port.

use std::{
    env::{self, VarError},
    net::{Ipv4Addr, SocketAddr},
};

use anyhow::Context;
use tokio::net::TcpListener;

const DEFAULT_PORT: u16 = 3000;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let port = port_from_env()?;
    let addr = SocketAddr::from((Ipv4Addr::LOCALHOST, port));
    let listener = TcpListener::bind(addr).await.with_context(|| {
        format!("could not listen on {addr}; is another app using port {port}?")
    })?;

    println!("Your to-do app is running at http://{addr}  (press Ctrl+C to stop)");

    axum::serve(listener, rust_first_app::app())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("the web server stopped with an error")
}

fn port_from_env() -> anyhow::Result<u16> {
    match env::var("PORT") {
        Ok(raw) => raw
            .parse::<u16>()
            .with_context(|| format!("PORT must be a number from 0 to 65535, got {raw:?}")),
        Err(VarError::NotPresent) => Ok(DEFAULT_PORT),
        Err(err) => Err(err).context("PORT is set but is not valid text"),
    }
}

async fn shutdown_signal() {
    match tokio::signal::ctrl_c().await {
        Ok(()) => println!("\nShutting down. Bye!"),
        Err(err) => {
            eprintln!("could not listen for Ctrl+C ({err}); close this window to stop the server");
            // Completing here would stop the server at once, so wait forever instead.
            std::future::pending::<()>().await;
        }
    }
}
