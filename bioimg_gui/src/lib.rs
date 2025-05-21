#![warn(clippy::all, rust_2018_idioms)]

pub mod app;
mod result;
mod task;
mod widgets;
mod project_data;
#[cfg(not(target_arch="wasm32"))]
mod request;
