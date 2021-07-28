use std::collections::BTreeMap;
use std::collections::HashSet;

use fpm::flatpak_manifest::{
    FlatpakManifest, FlatpakModule, FlatpakModuleDescription, FlatpakSource, FlatpakSourceDescription,
};

fn main() {
    fpm::logger::init();
    let db = fpm::db::Database::get_database();

    let mut app_ids_to_sources: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut sources_repos_count: BTreeMap<String, i64> = BTreeMap::new();
    let mut sources_repos_with_manifests_count: BTreeMap<String, i64> = BTreeMap::new();
    let mut sources_manifests_count: BTreeMap<String, i64> = BTreeMap::new();
    let mut sources_repos_with_modules_count: BTreeMap<String, i64> = BTreeMap::new();
    let mut sources_modules_count: BTreeMap<String, i64> = BTreeMap::new();

    if db.indexed_projects.len() == 0 {
        panic!("There are no projects in the database!");
    }

    for (project_id, project) in &db.indexed_projects {
        // We're only interested in having stats for the projects supporting Flatpak.
        if !project.supports_flatpak() {
            continue;
        }

        log::info!("Processing project {}...", project_id);
        let repo_url = project.get_main_vcs_url();

        let repo_dir = match fpm::utils::clone_git_repo(&repo_url) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("Could not clone repo {}: {}", &repo_url, e);
                continue;
            }
        };

        for manifest_path in &project.flatpak_app_manifests {
            let absolute_manifest_path = repo_dir.to_string() + manifest_path;

            let flatpak_manifest = match FlatpakManifest::load_from_file(absolute_manifest_path.to_string()) {
                Some(m) => m,
                None => {
                    log::warn!(
                        "Could not parse Flatpak manifest at {}!!!",
                        absolute_manifest_path
                    );
                    continue;
                }
            };

            // app_ids.insert(flatpak_manifest.get_id());
        }

        for manifest_path in &project.flatpak_module_manifests {
            let absolute_manifest_path = repo_dir.to_string() + manifest_path;
            let module_description = FlatpakModuleDescription::load_from_file(absolute_manifest_path).unwrap();
        }
    }
}
