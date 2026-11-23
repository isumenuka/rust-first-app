//! # `rust_first_app`
//!
//! A tiny to-do list web app built with [axum](https://docs.rs/axum).
//!
//! The crate is split so each layer can be tested on its own:
//! - [`todos`] holds the domain logic: plain, synchronous and unit-tested.
//! - [`routes`] connects that logic to HTTP handlers.
//!
//! `src/main.rs` only opens a socket and serves [`app`].
//!
//! # Examples
//!
//! ```no_run
//! # async fn run() -> std::io::Result<()> {
//! let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
//! axum::serve(listener, rust_first_app::app()).await
//! # }
//! ```
#![deny(missing_docs)]

pub mod routes;
pub mod todos;

pub use routes::app;
