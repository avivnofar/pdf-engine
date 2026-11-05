//! pdf-engine-js — the PdfCraft engine as a JavaScript library (the pdf-engine fork).
//!
//! - [`core::Engine`] is plain Rust: bytes and JSON in, bytes and JSON out. It is what the tests
//!   run natively, and what any other Rust host (a server, a desktop tool) can embed directly.
//! - `web` (wasm32 only) is a thin `wasm-bindgen` layer over it: the `PdfEngine` class the npm
//!   package exports. It converts types and nothing else, so every behaviour is tested in `core`.
//!
//! Build the npm package with `wasm-pack build apps/pdf-engine-js --target web --release`
//! (see `FORK.md`).

#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

pub mod core;

#[cfg(target_arch = "wasm32")]
mod web;
