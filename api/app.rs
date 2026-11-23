//! Vercel entry point: serves the same axum app as a Vercel Function.
//!
//! `vercel.json` rewrites every URL to this function, and the router still
//! sees the original path, so `/` and `/api/todos` work as they do locally.

use tower::Layer;
use vercel_runtime::{Error, axum::VercelLayer, run};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let service = VercelLayer::new().layer(rust_first_app::app());
    run(service).await
}
