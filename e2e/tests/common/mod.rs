//! Common test utilities and fixtures for E2E tests.

#![allow(dead_code)]

use serde_json::Value;

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
pub fn unique_name(prefix: &str) -> String {
    let uuid = uuid::Uuid::new_v4();
    format!("{}-{}", prefix, &uuid.to_string()[..8])
}

/// Macro to generate test cases for both transports.
#[macro_export]
macro_rules! transport_tests {
    ($($name:ident: $test_fn:expr,)*) => {
        $(
            mod $name {
                use super::*;

                #[tokio::test]
                async fn stdio() {
                    common::init_tracing();
                    let test_fn = $test_fn;
                    test_fn(TransportKind::Stdio).await;
                }

                #[tokio::test]
                async fn http() {
                    common::init_tracing();
                    let test_fn = $test_fn;
                    test_fn(TransportKind::Http).await;
                }
            }
        )*
    };
}
