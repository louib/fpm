use std::collections::BTreeMap;
use std::collections::HashSet;
use std::path;

use fpm::flatpak_manifest::{FlatpakManifest, FlatpakModule, FlatpakModuleDescription};

fn main() {
    fpm::logger::init();
    let db = fpm::db::Database::get_database();

    if db.indexed_projects.len() == 0 {
        panic!("There are no projects in the database!");
    }

    let mut git_urls_domains: BTreeMap<String, i64> = BTreeMap::new();
    let mut archive_urls_domains: BTreeMap<String, i64> = BTreeMap::new();
    let mut all_urls_domains: BTreeMap<String, i64> = BTreeMap::new();
    let mut git_urls_count: i64 = 0;
    let mut archive_urls_count: i64 = 0;

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

            for module in &flatpak_manifest.get_all_modules_recursively() {
                let module_description = match &module {
                    FlatpakModule::Path(_) => continue,
                    FlatpakModule::Description(d) => d,
                };

                for git_url in module_description.get_all_git_urls() {
                    git_urls_count += 1;
                }
                for archive_url in module_description.get_all_archive_urls() {
                    archive_urls_count += 1;
                }
            }
        }

        for manifest_path in &project.flatpak_module_manifests {
            let absolute_manifest_path = repo_dir.to_string() + manifest_path;
            let module_description = FlatpakModuleDescription::load_from_file(absolute_manifest_path).unwrap();

            // FIXME this should also get the sub-modules recursively.
            for git_url in module_description.get_all_git_urls() {
                git_urls_count += 1;
            }
            for archive_url in module_description.get_all_archive_urls() {
                archive_urls_count += 1;
            }

            for module in module_description.get_all_modules_recursively() {
                let module_description = match module {
                    FlatpakModule::Description(d) => d,
                    FlatpakModule::Path(_) => continue,
                };

                for git_url in module_description.get_all_git_urls() {
                    git_urls_count += 1;
                }
                for archive_url in module_description.get_all_archive_urls() {
                    archive_urls_count += 1;
                }
            }
        }
    }

    println!("===== Domain stats =====");
    println!("Extracted {} git urls from the manifests", git_urls_count,);
    println!("Extracted {} archive urls from the manifests", archive_urls_count,);
    println!("=====================");
}
