//! Common test utilities and fixtures for E2E tests.

#![allow(dead_code)]

use serde_json::Value;
use std::time::Duration;

/// Maximum attempts when polling for resource readiness.
pub const MAX_POLL_ATTEMPTS: usize = 20;

/// Delay between poll attempts.
pub const POLL_DELAY: Duration = Duration::from_secs(1);

/// Initialize tracing for tests.
pub fn init_tracing() {
    use tracing_subscriber::{EnvFilter, fmt};

    let _ = fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .try_init();
}

/// Helper to extract an array from a JSON value.
pub fn as_array(value: &Value) -> &Vec<Value> {
    value.as_array().expect("Expected JSON array")
}

/// Helper to extract an object from a JSON value.
pub fn as_object(value: &Value) -> &serde_json::Map<String, Value> {
    value.as_object().expect("Expected JSON object")
}

/// Helper to check if a tool call was successful (no error in result).
pub fn assert_tool_success(result: &Value) {
    if let Some(error) = result.get("error") {
        panic!("Tool call failed with error: {:?}", error);
    }
}

/// Create a unique name for test resources.
/// Uses full 32-char UUID to avoid collisions across test runs.
pub fn unique_name(prefix: &str) -> String {
    let uuid = uuid::Uuid::new_v4();
    format!("{}-{}", prefix, uuid.to_string().replace('-', ""))
}
