//! Utility functions shared across the application.

use std::net::SocketAddr;
use tokio::net::TcpListener;
use tracing::warn;

/// Find an available port, starting from the preferred port.
///
/// This function attempts to find an available port using the following strategy:
/// 1. Try the preferred port first
/// 2. If unavailable, try the next 10 consecutive ports
/// 3. If all are unavailable, let the OS assign a random available port
///
/// # Arguments
/// * `host` - The host address to bind to (e.g., "127.0.0.1")
/// * `preferred` - The preferred port number to try first
///
/// # Returns
/// The available port number, or an error if no port could be found.
///
/// # Example
/// ```ignore
/// let port = find_available_port("127.0.0.1", 20289).await?;
/// println!("Using port: {}", port);
/// ```
pub async fn find_available_port(host: &str, preferred: u16) -> std::io::Result<u16> {
    // Try preferred port
    let addr: SocketAddr = format!("{}:{}", host, preferred)
        .parse()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?;

    if let Ok(listener) = TcpListener::bind(addr).await {
        drop(listener);
        return Ok(preferred);
    }

    // Try next 10 ports
    for offset in 1..=10 {
        let port = preferred.saturating_add(offset);
        let addr: SocketAddr = format!("{}:{}", host, port)
            .parse()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?;

        if let Ok(listener) = TcpListener::bind(addr).await {
            drop(listener);
            warn!(
                preferred,
                actual = port,
                "Preferred port unavailable, using alternate"
            );
            return Ok(port);
        }
    }

    // Let OS assign a port
    let addr: SocketAddr = format!("{}:0", host)
        .parse()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?;
    let listener = TcpListener::bind(addr).await?;
    let port = listener.local_addr()?.port();
    drop(listener);
    warn!(preferred, actual = port, "Using OS-assigned port");
    Ok(port)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_find_available_port_preferred() {
        // Use a high port that's likely available
        let preferred = 49152; // Start of dynamic/private port range
        let port = find_available_port("127.0.0.1", preferred).await.unwrap();
        assert!(port > 0);
        // Should get the preferred port or one close to it
        assert!(port >= preferred && port <= preferred + 11);
    }

    #[tokio::test]
    async fn test_find_available_port_fallback() {
        // Bind to a port first, then try to find an available one starting from it
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let bound_port = listener.local_addr().unwrap().port();

        // Try to find a port starting from the bound one - should get a different port
        let port = find_available_port("127.0.0.1", bound_port).await.unwrap();
        assert!(port > 0);
        // Port should be different since the preferred is taken
        assert_ne!(port, bound_port);

        drop(listener);
    }

    #[tokio::test]
    async fn test_find_available_port_invalid_host() {
        let result = find_available_port("invalid-host-format[", 8080).await;
        assert!(result.is_err());
    }
}
