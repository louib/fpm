use std::collections::BTreeMap;
use std::collections::HashSet;
use std::path;

use fpm::flatpak_manifest::{FlatpakManifest, FlatpakModule, FlatpakModuleDescription};

const SHARED_MODULES_URL: &str = "https://github.com/flathub/shared-modules.git";

fn main() {
    fpm::logger::init();
    let db = fpm::db::Database::get_database();

    let mut shared_module_names: Vec<String> = vec![];
    let mut manifests_count: i64 = 0;
    let mut manifests_using_shared_modules_count: i64 = 0;
    let mut manifests_using_local_shared_modules_count: i64 = 0;
    let mut modules_count: i64 = 0;
    let mut path_modules_count: i64 = 0;
    let mut shared_modules_count: i64 = 0;

    if db.indexed_projects.len() == 0 {
        panic!("There are no projects in the database!");
    }

    let shared_modules_dir = match fpm::utils::clone_git_repo(SHARED_MODULES_URL) {
        Ok(d) => d,
        Err(e) => panic!("Could not clone shared modules repo: {}", e),
    };
    let shared_modules_file_paths = match fpm::utils::get_all_paths(path::Path::new(&shared_modules_dir)) {
        Ok(paths) => paths,
        Err(e) => panic!("Could not get file paths for shared modules dir: {}", e),
    };
    for file_path in &shared_modules_file_paths {
        if !file_path.is_file() {
            continue;
        }

        let file_path = match file_path.to_str() {
            Some(f) => f,
            None => continue,
        };

        if let Some(flatpak_module) = FlatpakModuleDescription::load_from_file(file_path.to_string()) {
            let flatpak_module_name = file_path.split("/").last().unwrap();
            shared_module_names.push(flatpak_module_name.to_string());
        }
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

            manifests_count += 1;
            let mut manifest_uses_shared_modules = false;
            let mut manifest_uses_local_shared_modules = false;

            // We're only looking at top-level modules on purpose here. We assume that this is the
            // main location for shared modules.
            for module in &flatpak_manifest.modules {
                modules_count += 1;

                let module_path = match module {
                    FlatpakModule::Description(_) => continue,
                    FlatpakModule::Path(p) => p,
                };

                path_modules_count += 1;

                let module_name = module_path.split("/").last().unwrap();
                let is_shared_module = shared_module_names.contains(&module_name.to_string());
                if is_shared_module {
                    shared_modules_count += 1;
                    if module_path.contains("shared-modules") {
                        manifest_uses_shared_modules = true;
                    } else {
                        manifest_uses_local_shared_modules = true;
                    }
                }
            }

            if manifest_uses_shared_modules {
                manifests_using_shared_modules_count += 1;
            }
            if manifest_uses_local_shared_modules {
                manifests_using_local_shared_modules_count += 1;
            }
        }
    }

    println!("===== Shared modules stats =====");
    println!("Total shared modules count: {}", shared_module_names.len());
    println!("Total manifest count: {}", manifests_count);
    println!(
        "Manifests using shared modules: {:.2}% ({}/{})",
        (manifests_using_shared_modules_count as f64 / manifests_count as f64) * 100.0,
        manifests_using_shared_modules_count,
        manifests_count,
    );
    println!(
        "Manifests using locally copied shared modules: {:.2}% ({}/{})",
        (manifests_using_local_shared_modules_count as f64 / manifests_count as f64) * 100.0,
        manifests_using_local_shared_modules_count,
        manifests_count,
    );

    println!("Total top-level module count: {}", modules_count);
    println!(
        "Path modules: {:.2}% ({}/{})",
        (path_modules_count as f64 / modules_count as f64) * 100.0,
        path_modules_count,
        modules_count,
    );
    println!(
        "Shared modules: {:.2}% ({}/{})",
        (shared_modules_count as f64 / modules_count as f64) * 100.0,
        shared_modules_count,
        modules_count,
    );
    println!("=====================");
}
