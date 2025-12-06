//! Transport layer tests
//!
//! Tests for HTTP/SSE configuration and basic functionality.

use tanuki_mcp::transport::HttpConfig;
use std::net::SocketAddr;

#[test]
fn test_http_config_default() {
    let config = HttpConfig::default();

    assert_eq!(config.bind, SocketAddr::from(([127, 0, 0, 1], 20289)));
    assert_eq!(config.sse_path, "/sse");
    assert_eq!(config.post_path, "/message");
}

#[test]
fn test_http_config_new() {
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    let config = HttpConfig::new(addr);

    assert_eq!(config.bind, addr);
    assert_eq!(config.sse_path, "/sse");
    assert_eq!(config.post_path, "/message");
}

#[test]
fn test_http_config_from_host_port() {
    let config = HttpConfig::from_host_port("127.0.0.1", 9000).unwrap();

    assert_eq!(config.bind.port(), 9000);
    assert_eq!(config.bind.ip().to_string(), "127.0.0.1");
}

#[test]
fn test_http_config_from_host_port_ipv6() {
    // IPv6 addresses need brackets in the format string for parsing
    let config = HttpConfig::from_host_port("[::1]", 8080).unwrap();

    assert_eq!(config.bind.port(), 8080);
    assert!(config.bind.ip().is_ipv6());
}

#[test]
fn test_http_config_from_host_port_invalid() {
    let result = HttpConfig::from_host_port("not-an-ip", 8080);
    assert!(result.is_err());
}

#[test]
fn test_http_config_clone() {
    let config1 = HttpConfig::new(SocketAddr::from(([127, 0, 0, 1], 3000)));
    let config2 = config1.clone();

    assert_eq!(config1.bind, config2.bind);
    assert_eq!(config1.sse_path, config2.sse_path);
    assert_eq!(config1.post_path, config2.post_path);
}

#[test]
fn test_http_config_debug() {
    let config = HttpConfig::default();
    let debug_str = format!("{:?}", config);

    assert!(debug_str.contains("HttpConfig"));
    assert!(debug_str.contains("127.0.0.1:20289"));
    assert!(debug_str.contains("/sse"));
    assert!(debug_str.contains("/message"));
}

// Note: Full HTTP/SSE integration tests would require starting a real server
// and making HTTP requests, which is more complex. The basic config tests
// above verify the configuration layer works correctly.

// ============================================================================
// Async Transport Tests (require tokio runtime)
// ============================================================================

#[tokio::test]
async fn test_http_server_config_creation() {
    // Verify we can create valid server configurations
    let configs = vec![
        HttpConfig::new(SocketAddr::from(([127, 0, 0, 1], 0))), // Ephemeral port
        HttpConfig::from_host_port("127.0.0.1", 0).unwrap(),
        HttpConfig::default(),
    ];

    for config in configs {
        // Should be able to create configs without panic
        assert!(!config.sse_path.is_empty());
        assert!(!config.post_path.is_empty());
    }
}
