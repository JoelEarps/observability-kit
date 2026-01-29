//! Deserialization support for creating registries from configuration files.
//!
//! This module provides functionality to deserialize metric definitions from
//! JSON or YAML files and create configured registries with optimized performance.

#[cfg(any(feature = "json-config", feature = "yaml-config"))]
pub mod config;

pub mod errors;
#[cfg(any(feature = "json-config", feature = "yaml-config"))]
pub mod registry;
#[cfg(any(feature = "json-config", feature = "yaml-config"))]
pub mod loaders;

#[cfg(any(feature = "json-config", feature = "yaml-config"))]
pub use config::{MetricConfig, RegistryConfig};

pub use errors::DeserializeError;

#[cfg(any(feature = "json-config", feature = "yaml-config"))]
pub use registry::ConfiguredRegistry;

#[cfg(any(feature = "json-config", feature = "yaml-config"))]
pub use loaders::*;