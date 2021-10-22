use std::collections::BTreeMap;
use std::fs;
use std::path;

use serde::{Deserialize, Serialize};

// Re-using the git cache directory so that the user doesn't have to worry
// about ignoring an additional directory.
pub const DEFAULT_CACHE_DIR: &str = ".git/";
pub const DEFAULT_CONFIG_FILE_NAME: &str = ".fpm-config.yaml";

#[derive(Deserialize, Serialize, Debug, Default)]
#[serde(default)]
pub struct WorkspaceConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_workspace: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_build: Option<String>,

    pub workspaces: BTreeMap<String, String>,
}

pub fn write_config(config: &WorkspaceConfig) -> Result<WorkspaceConfig, String> {
    let cache_dir = path::Path::new(DEFAULT_CACHE_DIR);
    if !cache_dir.is_dir() {
        match fs::create_dir(cache_dir) {
            Ok(_) => {}
            Err(e) => return Err(e.to_string()),
        };
    }

    let config_content = match serde_yaml::to_string(&config) {
        Ok(m) => m,
        Err(e) => return Err(format!("Failed to dump the config {}", e)),
    };

    let config_path = DEFAULT_CACHE_DIR.to_owned() + DEFAULT_CONFIG_FILE_NAME;
    let config_path = path::Path::new(&config_path);
    match fs::write(config_path, config_content) {
        Ok(m) => m,
        Err(e) => {
            return Err(format!(
                "Failed to write the config file at {}: {}",
                config_path.to_str().unwrap_or(""),
                e
            ))
        }
    };

    read_config()
}

pub fn read_config() -> Result<WorkspaceConfig, String> {
    // Make that more robust maybe?
    let config_path = DEFAULT_CACHE_DIR.to_owned() + DEFAULT_CONFIG_FILE_NAME;
    let config_path = path::Path::new(&config_path);
    let config_content = match fs::read_to_string(config_path) {
        Ok(m) => m,
        Err(e) => {
            return Err(format!(
                "Failed to read the config file at {}: {}.",
                config_path.to_str().unwrap_or(""),
                e,
            ))
        }
    };

    let config: WorkspaceConfig = match serde_yaml::from_str(&config_content) {
        Ok(m) => m,
        Err(e) => {
            return Err(format!(
                "Failed to parse the config file at {}: {}.",
                config_path.to_str().unwrap_or(""),
                e
            ))
        }
    };
    Ok(config)
}

pub fn get_manifest_path() -> Result<String, String> {
    let config = match read_or_init_config() {
        Ok(c) => c,
        Err(e) => return Err(format!("Could not load or init config: {}", e)),
    };

    let workspace_name = match &config.current_workspace {
        Some(w) => w,
        None => {
            return Err(format!(
                "Not currently in a workspace. Use `ls` to list the available workspaces and manifests."
            ))
        }
    };

    if !config.workspaces.contains_key(workspace_name) {
        return Err(format!(
            "Workspace {} does not exist. Use `ls` to list the available workspaces and manifests.",
            workspace_name
        ));
    }

    return Ok(config.workspaces.get(workspace_name).unwrap().to_string());
}

pub fn get_manifest() -> Result<flatpak_rs::flatpak_manifest::FlatpakManifest, String> {
    let path = match get_manifest_path() {
        Ok(p) => p,
        Err(e) => return Err(e),
    };
    log::debug!("Loading manifest at {}.", &path);

    flatpak_rs::flatpak_manifest::FlatpakManifest::load_from_file(path.to_string())
}

pub fn read_or_init_config() -> Result<WorkspaceConfig, String> {
    match read_config() {
        Ok(config) => Ok(config),
        Err(_) => match write_config(&WorkspaceConfig::default()) {
            Ok(c) => return Ok(c),
            Err(e) => return Err(e),
        },
    }
}
