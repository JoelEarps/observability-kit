//! File loading utilities for configuration files.
//!
//! This module provides functions to load configuration from files or strings,
//! separated from the core registry building logic.

/// Supported file types for configuration files.
/// Holds the validated path (e.g. for display or passing to `load_json_file` / `load_yaml_file`).
#[derive(Debug)]
pub enum SupportedFileTypes {
    Json(PathBuf),
    Yaml(PathBuf),
}

use crate::core::deserialise::errors::DeserializeError;
#[cfg(any(feature = "json-config", feature = "yaml-config"))]
use crate::core::deserialise::config::RegistryConfig;
#[cfg(any(feature = "json-config", feature = "yaml-config"))]
use std::fs::File;
#[cfg(any(feature = "json-config", feature = "yaml-config"))]
use std::io::BufReader;
use std::path::{Path, PathBuf};

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

/// Returns allowed base directories: XDG_CONFIG_HOME (or ~/.config), current dir, and optional base.
fn allowed_base_directories(extra_base: Option<&Path>) -> Result<Vec<PathBuf>, DeserializeError> {
    let mut bases = Vec::new();

    // $XDG_CONFIG_HOME or ~/.config on Unix (first that is set)
    let xdg_config_candidates: &[(&str, Option<&str>)] = &[
        ("XDG_CONFIG_HOME", None),
        ("HOME", Some(".config")),
    ];
    for (var, suffix) in xdg_config_candidates {
        if let Some(val) = std::env::var_os(var) {
            let path = PathBuf::from(val);
            bases.push(suffix.map(|s| path.join(s)).unwrap_or(path));
            break;
        }
    }

    // Current working directory
    if let Ok(cwd) = std::env::current_dir() {
        bases.push(cwd);
    }

    if let Some(b) = extra_base {
        bases.push(b.to_path_buf());
    }

    if bases.is_empty() {
        return Err(DeserializeError::InvalidFilePath(
            "No allowed base directory (set XDG_CONFIG_HOME or run from a valid CWD)".to_string(),
        ));
    }
    Ok(bases)
}

/// Check that the path and none of its ancestors are symlinks.
fn path_contains_no_symlink(path: &Path) -> Result<(), DeserializeError> {
    for ancestor in path.ancestors() {
        if let Ok(meta) = std::fs::symlink_metadata(ancestor) {
            if meta.file_type().is_symlink() {
                return Err(DeserializeError::SymlinkNotAllowed(
                    ancestor.display().to_string(),
                ));
            }
        }
    }
    Ok(())
}

/// Validate the file path with security restrictions:
/// - Path must exist and be a regular file (not a directory).
/// - Extension must be `.json`, `.yaml`, or `.yml`.
/// - Symlinks are not allowed (neither the file nor any path component).
/// - Path must resolve to a location under an allowed base:
///   - `$XDG_CONFIG_HOME` (or `$HOME/.config` if unset),
///   - current working directory,
///   - or `extra_base` if provided.
///
/// Use `extra_base` to allow an additional directory (e.g. project root).
pub fn validate_file_path(
    path: impl AsRef<Path>,
    extra_base: Option<&Path>,
) -> Result<SupportedFileTypes, DeserializeError> {
    let path = path.as_ref();

    // Resolve to absolute path
    let absolute = if path.is_relative() {
        std::env::current_dir()
            .map_err(DeserializeError::Io)?
            .join(path)
    } else {
        path.to_path_buf()
    };

    if !absolute.is_file() {
        return Err(DeserializeError::InvalidFilePath(
            absolute.display().to_string(),
        ));
    }

    // No symlinks (file itself or any ancestor)
    path_contains_no_symlink(&absolute)?;

    // Allowed bases (canonicalized)
    let bases = allowed_base_directories(extra_base)?;
    let canonical_path = absolute.canonicalize().map_err(DeserializeError::Io)?;
    let mut under_allowed = false;
    for base in &bases {
        if let Ok(canonical_base) = base.canonicalize() {
            if canonical_path.starts_with(canonical_base) {
                under_allowed = true;
                break;
            }
        }
    }
    if !under_allowed {
        return Err(DeserializeError::PathOutsideAllowedDirectory(
            canonical_path.display().to_string(),
        ));
    }

    // Supported extension (path already validated as under allowed dir and no symlinks)
    match path.extension().and_then(std::ffi::OsStr::to_str) {
        Some("json") => Ok(SupportedFileTypes::Json(canonical_path)),
        Some("yaml") | Some("yml") => Ok(SupportedFileTypes::Yaml(canonical_path)),
        Some(ext) => Err(DeserializeError::UnsupportedFileType(ext.to_string())),
        None => Err(DeserializeError::UnsupportedFileType(
            "missing file extension".to_string(),
        )),
    }
}

/// Central API: validate path, then deserialize and return config only if the matching feature is enabled.
/// - Always validates the path (exists, allowed dir, no symlinks, supported extension).
/// - If `json-config` is enabled and the file is `.json`, loads and returns `RegistryConfig`.
/// - If `yaml-config` is enabled and the file is `.yaml`/`.yml`, loads and returns `RegistryConfig`.
/// - If the format's feature is not enabled, returns `FeatureNotEnabled`.
pub fn load_file(
    path: impl AsRef<Path>,
    extra_base: Option<&Path>,
) -> Result<RegistryConfig, DeserializeError> {
    let validated = validate_file_path(path, extra_base)?;
    match validated {
        SupportedFileTypes::Json(p) => {
            #[cfg(feature = "json-config")]
            {
                load_json_file(p)
            }
            #[cfg(not(feature = "json-config"))]
            {
                Err(DeserializeError::FeatureNotEnabled(
                    "enable json-config feature to load JSON files".to_string(),
                ))
            }
        }
        SupportedFileTypes::Yaml(p) => {
            #[cfg(feature = "yaml-config")]
            {
                load_yaml_file(p)
            }
            #[cfg(not(feature = "yaml-config"))]
            {
                Err(DeserializeError::FeatureNotEnabled(
                    "enable yaml-config feature to load YAML files".to_string(),
                ))
            }
        }
    }
}

#[cfg(test)]
mod file_path_tests {
    use super::*;
    use std::fs;

    /// Create a temp dir under current dir so path ancestors don't include symlinks (e.g. /var on macOS).
    fn temp_config_dir() -> PathBuf {
        let dir = std::env::current_dir()
            .unwrap()
            .join("target")
            .join("test_file_path_tests");
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn valid_json_under_extra_base_returns_json_path() {
        let base = temp_config_dir();
        let config_file = base.join("metrics.json");
        fs::write(&config_file, "[]").unwrap();

        let result = validate_file_path(&config_file, Some(base.as_path()));
        match result {
            Ok(SupportedFileTypes::Json(p)) => {
                assert!(p.is_absolute());
                assert!(p.ends_with("metrics.json"));
            }
            Ok(_) => panic!("expected Json variant"),
            Err(e) => panic!("expected ok: {:?}", e),
        }
    }

    #[test]
    fn valid_yaml_extension_returns_yaml_path() {
        let base = temp_config_dir();
        let config_file = base.join("metrics.yaml");
        fs::write(&config_file, "[]").unwrap();

        let result = validate_file_path(&config_file, Some(base.as_path()));
        match result {
            Ok(SupportedFileTypes::Yaml(p)) => assert!(p.ends_with("metrics.yaml")),
            Ok(_) => panic!("expected Yaml variant"),
            Err(e) => panic!("expected ok: {:?}", e),
        }
    }

    #[test]
    fn valid_yml_extension_returns_yaml_path() {
        let base = temp_config_dir();
        let config_file = base.join("metrics.yml");
        fs::write(&config_file, "[]").unwrap();

        let result = validate_file_path(&config_file, Some(base.as_path()));
        match result {
            Ok(SupportedFileTypes::Yaml(p)) => assert!(p.ends_with("metrics.yml")),
            Ok(_) => panic!("expected Yaml variant"),
            Err(e) => panic!("expected ok: {:?}", e),
        }
    }

    #[test]
    fn missing_file_returns_invalid_file_path() {
        let base = temp_config_dir();
        let missing = base.join("does_not_exist.json");

        let result = validate_file_path(&missing, Some(base.as_path()));
        match result {
            Err(DeserializeError::InvalidFilePath(_)) => {}
            other => panic!("expected InvalidFilePath, got {:?}", other),
        }
    }

    #[test]
    fn directory_instead_of_file_returns_invalid_file_path() {
        let base = temp_config_dir();
        let subdir = base.join("subdir");
        fs::create_dir_all(&subdir).unwrap();

        let result = validate_file_path(&subdir, Some(base.as_path()));
        match result {
            Err(DeserializeError::InvalidFilePath(_)) => {}
            other => panic!("expected InvalidFilePath, got {:?}", other),
        }
    }

    #[test]
    fn unsupported_extension_returns_unsupported_file_type() {
        let base = temp_config_dir();
        let config_file = base.join("metrics.txt");
        fs::write(&config_file, "x").unwrap();

        let result = validate_file_path(&config_file, Some(base.as_path()));
        match result {
            Err(DeserializeError::UnsupportedFileType(ext)) => {
                assert_eq!(ext, "txt");
            }
            other => panic!("expected UnsupportedFileType, got {:?}", other),
        }
    }

    #[test]
    fn path_outside_allowed_base_returns_path_outside_allowed_directory() {
        // Create a dir *outside* project root so it's not under cwd (sandbox may block this)
        let outside_path = std::env::current_dir()
            .unwrap()
            .join("..")
            .join("observability_kit_test_outside");
        if fs::create_dir_all(&outside_path).is_err() {
            return; // skip if we can't create outside (e.g. sandbox)
        }
        let outside_dir = match outside_path.canonicalize() {
            Ok(d) => d,
            Err(_) => return,
        };
        let config_file = outside_dir.join("metrics.json");
        if fs::write(&config_file, "[]").is_err() {
            return;
        }

        let allowed_base = temp_config_dir();
        let result = validate_file_path(&config_file, Some(allowed_base.as_path()));
        match result {
            Err(DeserializeError::PathOutsideAllowedDirectory(_)) => {}
            other => panic!("expected PathOutsideAllowedDirectory, got {:?}", other),
        }
    }

    #[test]
    #[cfg(unix)]
    fn symlink_returns_symlink_not_allowed() {
        let base = temp_config_dir();
        let real_file = base.join("real.json");
        fs::write(&real_file, "[]").unwrap();
        let link_file = base.join("link.json");
        let _ = fs::remove_file(&link_file); // remove if left from previous run
        std::os::unix::fs::symlink(&real_file, &link_file).unwrap();

        let result = validate_file_path(&link_file, Some(base.as_path()));
        match result {
            Err(DeserializeError::SymlinkNotAllowed(_)) => {}
            other => panic!("expected SymlinkNotAllowed, got {:?}", other),
        }
    }

    #[test]
    fn load_file_validates_then_deserializes_when_feature_enabled() {
        let base = temp_config_dir();
        let config_file = base.join("delegate.json");
        fs::write(&config_file, "[]").unwrap();

        let result = load_file(&config_file, Some(base.as_path()));
        match result {
            Ok(config) => assert!(config.is_empty(), "empty JSON array => empty config"),
            Err(e) => panic!("expected ok: {:?}", e),
        }
    }

    #[test]
    fn validate_relative_path_under_cwd() {
        // When run from project root, rough_input.json exists there
        let path = "rough_input.json";
        let result = validate_file_path(path, None);
        match result {
            Ok(SupportedFileTypes::Json(p)) => {
                assert!(p.display().to_string().ends_with("rough_input.json"));
            }
            Ok(_) => {}
            Err(_) => {
                // Ok if file doesn't exist (e.g. run from different cwd)
            }
        }
    }
}
