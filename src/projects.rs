use std::collections::HashSet;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default)]
pub struct SoftwareProject {
    // Project ids are based on the reverse DNS notation, and
    // are either derived from build manifests found in the project
    // using the same reverse DNS notation, or from the git urls
    // associated with the project.
    pub id: String,

    // Common name of the software project.
    pub name: String,

    // Description of the software project.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    pub web_urls: HashSet<String>,

    pub vcs_urls: HashSet<String>,

    // A list of the paths of known flatpak app manifests found
    // in the project's repository.
    pub flatpak_app_manifests: HashSet<String>,

    // A list of the paths of known flatpak module definition manifests found
    // in the project's repository.
    pub flatpak_module_manifests: HashSet<String>,

    // A list of the sources from which a project was discovered.
    pub sources: HashSet<String>,

    // All the build systems that are know to be supported by the project.
    pub build_systems: HashSet<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_branch: Option<String>,

    // The root git commit hashes associated with the project. This is used
    // for project de-duplication, in the case a project has multiple remote
    // git repositories.
    pub root_hashes: Vec<String>,
}
impl SoftwareProject {
    pub fn merge(&mut self, other_project: &SoftwareProject) {
        for web_url in &other_project.web_urls {
            self.web_urls.insert(web_url.to_string());
        }
        for vcs_url in &other_project.vcs_urls {
            self.vcs_urls.insert(vcs_url.to_string());
        }
        for app_manifest in &other_project.flatpak_app_manifests {
            self.flatpak_app_manifests.insert(app_manifest.to_string());
        }
        for module_manifest in &other_project.flatpak_module_manifests {
            self.flatpak_module_manifests.insert(module_manifest.to_string());
        }
        for build_system in &other_project.build_systems {
            self.build_systems.insert(build_system.to_string());
        }
        // TODO validate that the root hashes are the same!
    }

    pub fn supports_flatpak(&self) -> bool {
        return !self.flatpak_app_manifests.is_empty() || !self.flatpak_module_manifests.is_empty();
    }

    pub fn get_main_vcs_url(&self) -> String {
        for vcs_url in &self.vcs_urls {
            return vcs_url.to_string();
        }
        panic!();
    }
}
