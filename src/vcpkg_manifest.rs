use serde::{Deserialize, Serialize};

/// Main structure for a vcpkg manifest.
/// See https://vcpkg.readthedocs.io/en/latest/specifications/manifests/
#[derive(Clone)]
#[derive(Deserialize)]
#[derive(Serialize)]
#[derive(Debug)]
#[derive(Default)]
#[serde(rename_all = "kebab-case")]
#[serde(default)]
pub struct VCPKGManifest {}
impl VCPKGManifest {
    pub fn file_path_matches(path: &str) -> bool {
        path.ends_with("vcpkg.json")
    }
}

#[cfg(test)]
mod manifest_tests {
    use super::*;

    #[test]
    pub fn test_parse_single_source() {}
}
