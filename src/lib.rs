// SPDX-License-Identifier: MIT OR Apache-2.0

//! Bitcoin Core RPC client library.
//!
//! This crate provides a Rust client for interacting with Bitcoin Core's JSON-RPC interface.
//! It supports multiple authentication methods and provides a type-safe interface for
//! making RPC calls to a Bitcoin Core daemon.

mod client;
mod error;

pub use client::*;
pub use error::*;

#[cfg(all(feature = "28_0", not(feature = "29_0")))]
pub mod v28;

pub use corepc_types;
pub use jsonrpc;
