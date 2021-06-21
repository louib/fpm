use std::collections::BTreeMap;
use std::collections::HashSet;

use fpm::flatpak_manifest::{
    FlatpakManifest, FlatpakModule, FlatpakModuleDescription, FlatpakSource, FlatpakSourceDescription,
};

fn main() {
    fpm::logger::init();
    let db = fpm::db::Database::get_database();

    if db.indexed_projects.len() == 0 {
        panic!("There are no projects in the database!");
    }

    let mut all_git_urls_from_manifests: HashSet<String> = HashSet::new();
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

            for module in &flatpak_manifest.modules {
                let module_description = match &module {
                    FlatpakModule::Path(_) => continue,
                    FlatpakModule::Description(d) => d,
                };

                for source in &module_description.sources {
                    if source.get_type_name() != "git" {
                        continue;
                    }

                    let source_description = match &source {
                        FlatpakSource::Path(_) => continue,
                        FlatpakSource::Description(d) => d,
                    };

                    let git_url = match &source_description.url {
                        Some(u) => u,
                        None => continue,
                    };
                    all_git_urls_from_manifests.insert(git_url.to_string());

                }
            }
        }

        for manifest_path in &project.flatpak_module_manifests {
            let absolute_manifest_path = repo_dir.to_string() + manifest_path;
            let flatpak_module = FlatpakModuleDescription::load_from_file(absolute_manifest_path).unwrap();
        }
    }
}
