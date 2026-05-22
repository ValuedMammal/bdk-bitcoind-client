// SPDX-License-Identifier: MIT OR Apache-2.0

//! Connecting to a Bitcoin Core RPC hidden service over Tor.
//!
//! Self-hosting stacks (Umbrel, Start9, RaspiBlitz, etc.) commonly expose
//! Bitcoin Core's RPC interface as a Tor hidden service rather than forwarding
//! a port or setting up a VPN. The only way to reach an `.onion` address is
//! through a SOCKS5 proxy connected to Tor.
//!
//! The library is intentionally agnostic about HTTP clients, proxies, and async
//! runtimes — it exposes [`jsonrpc::Transport`] as the integration point. This
//! example shows how to implement that trait against a SOCKS5-capable HTTP client
//! and pass it to [`bitreq::Client::with_transport`] to get the full RPC method
//! surface.
//!
//! The same pattern works for any SOCKS5 proxy; Tor is the common case.
//!
//! # Steps
//!
//! 1. Choose an HTTP client with SOCKS5 support (here: `reqwest` blocking).
//! 2. Implement [`jsonrpc::Transport`] for a thin wrapper around that client.
//! 3. Call [`bitreq::Client::with_transport`] — that's it.
//!
//! # Usage
//!
//! ```text
//! BITCOIND_URL=http://<your-node>.onion:8332 \
//! BITCOIND_AUTH=user:password \
//! SOCKS5_PROXY=127.0.0.1:9050 \
//! cargo run --example tor --features bitreq,29_0
//! ```
//!
//! # Additional dependency
//!
//! This example requires `reqwest` in your `Cargo.toml`:
//!
//! ```toml
//! reqwest = { version = "0.12", features = ["blocking", "socks", "json"] }
//! ```

use std::fmt;

use bdk_bitcoind_client::bitreq::Client;
use bdk_bitcoind_client::jsonrpc::{self, Request, Response, Transport};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url =
        std::env::var("BITCOIND_URL").unwrap_or_else(|_| "http://127.0.0.1:8332".to_string());
    let auth =
        std::env::var("BITCOIND_AUTH").unwrap_or_else(|_| "user:password".to_string());
    let proxy =
        std::env::var("SOCKS5_PROXY").unwrap_or_else(|_| "127.0.0.1:9050".to_string());

    let (user, pass) = auth.split_once(':').expect("BITCOIND_AUTH must be 'user:password'");

    let transport = Socks5Transport::new(&url, &proxy, user, pass)?;
    let client = Client::with_transport(transport);

    let count = client.get_block_count()?;
    let hash = client.get_best_block_hash()?;

    println!("Block count : {count}");
    println!("Best block  : {hash}");

    Ok(())
}

/// A SOCKS5-capable transport for the Bitcoin Core JSON-RPC API.
///
/// Wraps a [`reqwest::blocking::Client`] configured with a SOCKS5 proxy and
/// implements [`jsonrpc::Transport`] so it can be passed to
/// [`Client::with_transport`].
///
/// Note: `socks5h://` (with the `h`) routes DNS resolution through the proxy
/// as well. This matters for Tor — `socks5://` would resolve the hostname
/// locally before connecting, leaking DNS queries.
struct Socks5Transport {
    url: String,
    client: reqwest::blocking::Client,
    user: String,
    pass: String,
}

impl Socks5Transport {
    fn new(url: &str, proxy_addr: &str, user: &str, pass: &str) -> Result<Self, reqwest::Error> {
        let proxy = reqwest::Proxy::all(format!("socks5h://{proxy_addr}"))?;
        let client = reqwest::blocking::Client::builder().proxy(proxy).build()?;
        Ok(Self {
            url: url.to_string(),
            client,
            user: user.to_string(),
            pass: pass.to_string(),
        })
    }
}

impl Transport for Socks5Transport {
    fn send_request(&self, req: Request) -> Result<Response, jsonrpc::Error> {
        self.client
            .post(&self.url)
            .basic_auth(&self.user, Some(&self.pass))
            .json(&req)
            .send()
            .and_then(|r| r.json())
            .map_err(|e| jsonrpc::Error::Transport(Box::new(e)))
    }

    fn send_batch(&self, reqs: &[Request]) -> Result<Vec<Response>, jsonrpc::Error> {
        self.client
            .post(&self.url)
            .basic_auth(&self.user, Some(&self.pass))
            .json(reqs)
            .send()
            .and_then(|r| r.json())
            .map_err(|e| jsonrpc::Error::Transport(Box::new(e)))
    }

    fn fmt_target(&self, f: &mut fmt::Formatter) -> fmt::Result { f.write_str(&self.url) }
}
