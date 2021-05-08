use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default)]
pub struct SoftwareProject {
    // Project ids are based on the reverse DNS notation, and
    // are either derived from build manifests found in the project
    // using the same reverse DNS notation, or from the git urls
    // associated to the project.
    pub id: String,
    pub name: String,
    // Basically a short description, or a title.
    pub summary: String,
    pub description: String,
    // TODO should be a HashSet instead
    pub web_urls: Vec<String>,
    // TODO should be a HashSet instead
    pub vcs_urls: Vec<String>,
    // Name of the artifacts that this project produces. Can be binaries, libraries or assets.
    // TODO should be a HashSet instead
    pub artifact_names: Vec<String>,
    // Name of the build systems seen on the project.
    // TODO should be a HashSet instead
    pub build_systems: Vec<String>,
    // TODO should be a HashSet instead
    pub maintainers: Vec<String>,
    pub default_branch: Option<String>,
    pub versions: Vec<String>,
    // TODO should be a HashSet instead
    pub keywords: Vec<String>,

    // The root git commit hashes associated with the project. This is used
    // for project de-duplication, in the case a project has multiple remote
    // git repositories.
    pub root_hashes: Vec<String>,
}
impl SoftwareProject {
    pub fn merge(&mut self, other_project: &SoftwareProject) {
        for build_system in &other_project.build_systems {
            self.build_systems.push(build_system.clone());
        }
    }
}
