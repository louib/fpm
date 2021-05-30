use std::collections::HashSet;
use std::collections::BTreeMap;
use std::path;
use std::fs;
use std::env;
use std::process::exit;

use fpm::flatpak_manifest::{FlatpakManifest, FlatpakSource, FlatpakModule, FlatpakModuleDescription};

fn main() {
    let db = fpm::db::Database::get_database();

    let mut sources_count: BTreeMap<String, i64> = BTreeMap::new();
    let mut sources_total_count: i64 = 0;
    let mut invalid_sources_count: i64 = 0;
    let mut empty_sources_count: i64 = 0;
    let mut modules_count: i64 = 0;
    let mut modules_sources_count: BTreeMap<i32, i64> = BTreeMap::new();
    let mut modules_buildsystems_count: BTreeMap<String, i64> = BTreeMap::new();
    let mut manifests_max_depth: BTreeMap<i32, i64> = BTreeMap::new();
    let mut manifests_count: i64 = 0;
    let mut extension_manifests_count: i64 = 0;
    let mut extensions_count: BTreeMap<String, i64> = BTreeMap::new();
    let mut no_extensions_count: i64 = 0;
    let mut patched_modules_count: i64 = 0;

    for (project_id, project) in &db.indexed_projects {
        let repo_url = project.get_main_vcs_url();

        let repo_dir = match fpm::utils::clone_git_repo(&repo_url) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("Could not clone repo {}: {}", &repo_url, e);
                continue;
            },
        };
        let repo_file_paths = match fpm::utils::get_all_paths(path::Path::new(&repo_dir)) {
            Ok(paths) => paths,
            Err(message) => {
                log::error!("Could not get the file paths for {} :sad: {}", repo_dir, message);
                continue;
            }
        };

        for file_path in &repo_file_paths {
            if !file_path.is_file() {
                continue;
            }

            let file_path = match file_path.to_str() {
                Some(f) => f,
                None => continue,
            };

            if file_path.contains(".git/") {
                continue;
            }


            if let Some(flatpak_manifest) = FlatpakManifest::load_from_file(file_path.to_string()) {
                manifests_count += 1;

                if flatpak_manifest.is_extension() {
                    extension_manifests_count += 1;
                }

                for extension_name in &flatpak_manifest.sdk_extensions {
                    let new_count = extensions_count.get(extension_name).unwrap_or(&0) + 1;
                    extensions_count.insert(extension_name.to_string(), new_count);
                }
                if flatpak_manifest.sdk_extensions.len() == 0 {
                    no_extensions_count += 1;
                }

                let manifest_depth = flatpak_manifest.get_max_depth();
                let new_count = manifests_max_depth.get(&manifest_depth).unwrap_or(&0) + 1;
                manifests_max_depth.insert(manifest_depth, new_count);

                for module in &flatpak_manifest.modules {
                    let module_description = match &module {
                        FlatpakModule::Path(_) => continue,
                        FlatpakModule::Description(d) => d,
                    };

                    // We're only counting inlined module descriptions for now.
                    modules_count += 1;
                    for url in module.get_all_repos_urls() {
                        println!("MODULE URL {}", url);
                    }

                    if module.is_patched() {
                        patched_modules_count += 1;
                    }

                    let module_sources_count = module.get_sources_count() as i32;
                    let new_sources_count = modules_sources_count.get(&module_sources_count).unwrap_or(&0) + 1;
                    modules_sources_count.insert(module_sources_count, new_sources_count);

                    let new_buildsystem_count = modules_buildsystems_count.get(&module_description.buildsystem).unwrap_or(&0) + 1;
                    modules_buildsystems_count.insert(module_description.buildsystem.to_string(), new_buildsystem_count);

                    for source in &module_description.sources {
                        sources_total_count += 1;

                        let source_type_name = source.get_type_name();
                        let new_count = sources_count.get(&source_type_name).unwrap_or(&0) + 1;
                        sources_count.insert(source_type_name, new_count);

                        if !source.type_is_valid() {
                            invalid_sources_count += 1;
                        }

                        if source.type_is_empty() {
                            empty_sources_count += 1;
                        }
                    }
                }

            }

            if let Some(flatpak_module) = FlatpakModuleDescription::load_from_file(file_path.to_string()) {
                modules_count += 1;

            }

        }
    }
    println!("Manifests:");
    for (depth, depth_count) in manifests_max_depth {
        println!("Depth {}: {} ({}/{})%", depth, (depth_count as f64 / manifests_count as f64) * 100.0, depth_count, manifests_count);
    }
    println!("Number of extension manifests: {}.", extension_manifests_count);
    for (extension_name, count) in extensions_count {
        println!("Extension {}: {} ({}/{})%", extension_name, (count as f64 / manifests_count as f64) * 100.0, count, manifests_count);
    }
    println!("Manifests with no SDK extensions: {} ({}/{})%", (no_extensions_count as f64 / manifests_count as f64) * 100.0, no_extensions_count, manifests_count);
    println!("\n");

    println!("Modules:");
    println!("Patched modules: {} ({}/{})%", (patched_modules_count as f64 / modules_count as f64) * 100.0, patched_modules_count, modules_count);
    for (source_count, count) in modules_sources_count {
        println!("Modules with {} source(s): {} ({}/{})%", source_count, (count as f64 / modules_count as f64) * 100.0, count, sources_total_count);
    }
    for (buildsystem, buildsystem_count) in modules_buildsystems_count {
        println!("Modules with buildsystem {}: {} ({}/{})%", buildsystem, (buildsystem_count as f64 / modules_count as f64) * 100.0, buildsystem_count, modules_count);
    }
    println!("\n");

    println!("Sources:");
    for (source_type, source_count) in sources_count {
        println!("{}: {} ({}/{})%", source_type, (source_count as f64 / sources_total_count as f64) * 100.0, source_count, sources_total_count);
    }
    println!("Sources with invalid type: {}.", invalid_sources_count);
    println!("Sources with empty type: {}.", empty_sources_count);

    fpm::logger::init();
}
