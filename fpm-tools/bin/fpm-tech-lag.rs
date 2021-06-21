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
    let mut all_archive_urls: HashSet<String> = HashSet::new();

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

            // FIXME we should get the modules recursively here!!!
            for module in &flatpak_manifest.modules {
                let module_description = match &module {
                    FlatpakModule::Path(_) => continue,
                    FlatpakModule::Description(d) => d,
                };

                for git_url in module_description.get_all_git_urls() {
                    all_git_urls_from_manifests.insert(git_url.to_string());
                }
                for archive_url in module_description.get_all_archive_urls() {
                    all_archive_urls.insert(archive_url.to_string());
                }
            }
        }

        for manifest_path in &project.flatpak_module_manifests {
            let absolute_manifest_path = repo_dir.to_string() + manifest_path;
            let module_description =
                FlatpakModuleDescription::load_from_file(absolute_manifest_path).unwrap();
            for git_url in module_description.get_all_git_urls() {
                all_git_urls_from_manifests.insert(git_url.to_string());
            }
            for archive_url in module_description.get_all_archive_urls() {
                all_archive_urls.insert(archive_url.to_string());
            }
        }

        log::info!(
            "Extracted {} git urls from the manifests",
            all_git_urls_from_manifests.len()
        );
        log::info!(
            "Extracted {} archive urls from the manifests",
            all_archive_urls.len()
        );
    }

    for archive_url in &all_archive_urls {
        if fpm::utils::get_semver_from_archive_url(&archive_url).is_none() {
            continue;
        }
        get_git_url_for_archive(archive_url, &all_git_urls_from_manifests);
    }
}

fn get_git_url_for_archive(archive_url: &str, candidate_git_urls: &HashSet<String>) -> Option<String> {
    if let Some(git_url) = fpm::utils::get_git_url_from_archive_url(archive_url) {
        return Some(git_url);
    }

    for git_url in candidate_git_urls {
        let git_url_matches = match git_url_matches_archive(git_url, archive_url) {
            Ok(r) => r,
            Err(e) => panic!("{}", e),
        };
        if git_url_matches {
            return Some(git_url.to_string());
        }
    }

    // TODO search in the archive for other potention git repositories.
    None
}

fn git_url_matches_archive(git_url: &str, archive_url: &str) -> Result<bool, String> {
    let archive_version = fpm::utils::get_semver_from_archive_url(&archive_url).unwrap();
    Ok(false)
}
