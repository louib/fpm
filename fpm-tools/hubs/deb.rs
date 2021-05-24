use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct DebianPackagesHub {}
impl DebianPackagesHub {
    pub fn get_modules_from_debian_repository(
        repo_name: &str,
        repo_sources_url: &str,
    ) -> Vec<fpm::flatpak_manifest::FlatpakModule> {
        log::info!("Getting debian repos from {} at {}", repo_name, repo_sources_url);
        vec![]
    }
}
