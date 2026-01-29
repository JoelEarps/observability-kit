//! File loading utilities for configuration files.
//!
//! This module provides functions to load configuration from files or strings,
//! separated from the core registry building logic.

use crate::core::deserialise::errors::DeserializeError;
#[cfg(any(feature = "json-config", feature = "yaml-config"))]
use crate::core::deserialise::config::RegistryConfig;
#[cfg(any(feature = "json-config", feature = "yaml-config"))]
use std::fs::File;
#[cfg(any(feature = "json-config", feature = "yaml-config"))]
use std::io::BufReader;

/// Load a registry configuration from a JSON file.
///
/// Uses `BufReader` for efficient file reading.
///
/// # Example
/// ```ignore
/// use observability_kit::core::deserialise::loaders::load_json_file;
///
/// let config = load_json_file("metrics.json")?;
/// ```
#[cfg(feature = "json-config")]
pub fn load_json_file(path: impl AsRef<std::path::Path>) -> Result<RegistryConfig, DeserializeError> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let config: RegistryConfig = serde_json::from_reader(reader)?;
    Ok(config)
}

/// Load a registry configuration from a JSON string.
///
/// # Example
/// ```ignore
/// use observability_kit::core::deserialise::loaders::load_json_str;
///
/// let json = r#"[{"metric_type": "Counter", ...}]"#;
/// let config = load_json_str(json)?;
/// ```
#[cfg(feature = "json-config")]
pub fn load_json_str(json: &str) -> Result<RegistryConfig, DeserializeError> {
    let config: RegistryConfig = serde_json::from_str(json)?;
    Ok(config)
}

/// Load a registry configuration from a YAML file.
///
/// Uses `BufReader` for efficient file reading.
///
/// # Example
/// ```ignore
/// use observability_kit::core::deserialise::loaders::load_yaml_file;
///
/// let config = load_yaml_file("metrics.yaml")?;
/// ```
#[cfg(feature = "yaml-config")]
pub fn load_yaml_file(path: impl AsRef<std::path::Path>) -> Result<RegistryConfig, DeserializeError> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let config: RegistryConfig = serde_yaml::from_reader(reader)?;
    Ok(config)
}

/// Load a registry configuration from a YAML string.
///
/// # Example
/// ```ignore
/// use observability_kit::core::deserialise::loaders::load_yaml_str;
///
/// let yaml = r#"
/// metrics:
///   - metric_type: Counter
///     title: test_counter
/// "#;
/// let config = load_yaml_str(yaml)?;
/// ```
#[cfg(feature = "yaml-config")]
pub fn load_yaml_str(yaml: &str) -> Result<RegistryConfig, DeserializeError> {
    let config: RegistryConfig = serde_yaml::from_str(yaml)?;
    Ok(config)
}
