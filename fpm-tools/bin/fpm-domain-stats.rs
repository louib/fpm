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
                    let domain = fpm::utils::url_to_domain(&git_url);
                    let new_count = git_urls_domains.get(&domain).unwrap_or(&0) + 1;
                    git_urls_domains.insert(domain.to_string(), new_count);
                }
                for archive_url in module_description.get_all_archive_urls() {
                    archive_urls_count += 1;
                    let domain = fpm::utils::url_to_domain(&archive_url);
                    let new_count = archive_urls_domains.get(&domain).unwrap_or(&0) + 1;
                    archive_urls_domains.insert(domain.to_string(), new_count);
                }
            }
        }

        for manifest_path in &project.flatpak_module_manifests {
            let absolute_manifest_path = repo_dir.to_string() + manifest_path;
            // We don't want to include the shared modules here, because otherwise we would be
            // counting multiple times the same modules.
            if absolute_manifest_path.contains("shared-modules") {
                continue;
            }

            let module_description = FlatpakModuleDescription::load_from_file(absolute_manifest_path).unwrap();

            for git_url in module_description.get_all_git_urls() {
                git_urls_count += 1;
                let domain = fpm::utils::url_to_domain(&git_url);
                let new_count = git_urls_domains.get(&domain).unwrap_or(&0) + 1;
                git_urls_domains.insert(domain.to_string(), new_count);
            }
            for archive_url in module_description.get_all_archive_urls() {
                archive_urls_count += 1;
                let domain = fpm::utils::url_to_domain(&archive_url);
                let new_count = archive_urls_domains.get(&domain).unwrap_or(&0) + 1;
                archive_urls_domains.insert(domain.to_string(), new_count);
            }

            for module in module_description.get_all_modules_recursively() {
                let module_description = match module {
                    FlatpakModule::Description(d) => d,
                    FlatpakModule::Path(_) => continue,
                };

                for git_url in module_description.get_all_git_urls() {
                    git_urls_count += 1;
                    let domain = fpm::utils::url_to_domain(&git_url);
                    let new_count = git_urls_domains.get(&domain).unwrap_or(&0) + 1;
                    git_urls_domains.insert(domain.to_string(), new_count);
                }
                for archive_url in module_description.get_all_archive_urls() {
                    archive_urls_count += 1;
                    let domain = fpm::utils::url_to_domain(&archive_url);
                    let new_count = archive_urls_domains.get(&domain).unwrap_or(&0) + 1;
                    archive_urls_domains.insert(domain.to_string(), new_count);
                }
            }
        }
    }

    println!("===== Domain stats =====");
    println!("Extracted {} git urls from the manifests", git_urls_count,);
    println!("Extracted {} archive urls from the manifests", archive_urls_count,);
    for (domain, count) in git_urls_domains {
        let percentage = (count as f64 / git_urls_count as f64) * 100.0;
        if percentage < 1.0 {
            // TODO merge those.
            continue;
        }
        println!(
            "Git URLS with domain {}: {:.2}% ({}/{})",
            domain,
            percentage,
            count,
            git_urls_count,
        );
    }
    println!("=====================");
}
