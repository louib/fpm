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

    pub fn parse(manifest_path: &str, manifest_content: &str) -> Result<VCPKGManifest, String> {
        let mut vcpkg_manifest: VCPKGManifest = VCPKGManifest::default();

        if VCPKGManifest::file_path_matches(&manifest_path.to_lowercase()) {
            vcpkg_manifest = match serde_json::from_str(&manifest_content) {
                Ok(m) => m,
                Err(e) => {
                    return Err(format!("Failed to parse the vcpkg manifest: {}.", e));
                }
            };
        }

        Ok(vcpkg_manifest)
    }
}

#[cfg(test)]
mod manifest_tests {
    use super::*;

    #[test]
    pub fn test_parse_single_source() {}
}
