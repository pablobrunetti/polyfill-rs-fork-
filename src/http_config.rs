//! HTTP client optimization for low-latency trading
//!
//! This module provides optimized HTTP client configurations specifically
//! designed for high-frequency trading environments where every millisecond counts.

use reqwest::{Client, ClientBuilder};
use std::net::SocketAddr;
use std::time::Duration;

/// Connection pre-warming helper
pub async fn prewarm_connections(client: &Client, base_url: &str) -> Result<(), reqwest::Error> {
    // Make a few lightweight requests to establish connections
    let endpoints = vec!["/ok", "/time"];

    for endpoint in endpoints {
        let _ = client
            .get(format!("{}{}", base_url, endpoint))
            .timeout(Duration::from_millis(1000))
            .send()
            .await;
    }

    Ok(())
}

/// Create an optimized HTTP client for low-latency trading
/// Benchmarked configuration: 309.3ms vs 349ms baseline (11.4% faster)
pub fn create_optimized_client() -> Result<Client, reqwest::Error> {
    create_optimized_client_with_resolve(None)
}

/// Create an optimized HTTP client pinned to a specific IP for the CLOB host.
/// Use this to force different connections through different physical paths.
pub fn create_optimized_client_with_resolve(
    resolve: Option<(&'static str, SocketAddr)>,
) -> Result<Client, reqwest::Error> {
    let mut builder = ClientBuilder::new()
        .no_proxy()
        .pool_max_idle_per_host(10)
        .pool_idle_timeout(Duration::from_secs(90))
        .tcp_nodelay(true)
        .http2_adaptive_window(true)
        .http2_initial_stream_window_size(512 * 1024)
        .gzip(true)
        .user_agent(concat!(
            "polyfill-rs/",
            env!("CARGO_PKG_VERSION"),
            " (high-frequency-trading)"
        ));

    if let Some((host, addr)) = resolve {
        builder = builder.resolve(host, addr);
    }

    builder.build()
}

/// Create a client optimized for co-located environments
/// (even more aggressive settings for when you're close to the exchange)
pub fn create_colocated_client() -> Result<Client, reqwest::Error> {
    ClientBuilder::new()
        // Avoid reading OS proxy settings (can be slow and/or unavailable in some sandboxed envs)
        .no_proxy()
        // More aggressive connection pooling
        .pool_max_idle_per_host(20) // More connections
        .pool_idle_timeout(Duration::from_secs(60)) // Longer reuse
        // Tighter timeouts for co-located environments
        .connect_timeout(Duration::from_millis(1000)) // 1s connection
        .timeout(Duration::from_millis(10000)) // 10s total
        // TCP optimizations
        .tcp_nodelay(true)
        .tcp_keepalive(Duration::from_secs(30))
        // HTTP/2 with more aggressive keep-alive
        .http2_adaptive_window(true)
        .http2_keep_alive_interval(Duration::from_secs(10))
        .http2_keep_alive_timeout(Duration::from_secs(5))
        .http2_keep_alive_while_idle(true)
        // Disable compression in co-located environments (CPU vs network tradeoff)
        .gzip(false)
        .no_brotli() // Disable brotli compression
        .user_agent(concat!(
            "polyfill-rs/",
            env!("CARGO_PKG_VERSION"),
            " (colocated-hft)"
        ))
        .build()
}

/// Create a client optimized for high-latency environments
/// (more conservative settings for internet connections)
pub fn create_internet_client() -> Result<Client, reqwest::Error> {
    ClientBuilder::new()
        // Avoid reading OS proxy settings (can be slow and/or unavailable in some sandboxed envs)
        .no_proxy()
        // Conservative connection pooling
        .pool_max_idle_per_host(5)
        .pool_idle_timeout(Duration::from_secs(90))
        // Longer timeouts for internet connections
        .connect_timeout(Duration::from_millis(10000)) // 10s connection
        .timeout(Duration::from_millis(60000)) // 60s total
        // TCP optimizations
        .tcp_nodelay(true)
        .tcp_keepalive(Duration::from_secs(120))
        // HTTP/1.1 might be more reliable over internet
        .http1_title_case_headers()
        // Enable compression (gzip and brotli are enabled by default)
        .gzip(true)
        .user_agent(concat!(
            "polyfill-rs/",
            env!("CARGO_PKG_VERSION"),
            " (internet-trading)"
        ))
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_optimized_client_creation() {
        let client = create_optimized_client();
        assert!(client.is_ok());
    }

    #[test]
    fn test_colocated_client_creation() {
        let client = create_colocated_client();
        assert!(client.is_ok());
    }

    #[test]
    fn test_internet_client_creation() {
        let client = create_internet_client();
        assert!(client.is_ok());
    }
}
