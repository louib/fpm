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

    pub description: String,

    // TODO should be a HashSet instead
    pub web_urls: Vec<String>,

    // TODO should be a HashSet instead
    pub vcs_urls: Vec<String>,

    // A list of the paths of known flatpak app manifests found
    // in the project's repository.
    pub flatpak_app_manifests: Vec<String>,

    // A list of the paths of known flatpak module definition manifests found
    // in the project's repository.
    pub flatpak_module_manifests: Vec<String>,

    // TODO should be a HashSet instead
    pub maintainers: Vec<String>,

    pub default_branch: Option<String>,

    // The root git commit hashes associated with the project. This is used
    // for project de-duplication, in the case a project has multiple remote
    // git repositories.
    pub root_hashes: Vec<String>,
}
impl SoftwareProject {
    pub fn merge(&mut self, other_project: &SoftwareProject) {
        for maintainer in &other_project.maintainers {
            self.maintainers.push(maintainer.to_string());
        }
    }
}
