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
                println!("MANIFEST MAX DEPTH {} {}", flatpak_manifest.get_max_depth(), file_path);

                for module in &flatpak_manifest.modules {
                    for url in module.get_all_repos_urls() {
                        println!("MODULE URL {}", url);
                    }

                    if let FlatpakModule::Description(d) = module {
                        for source in &d.sources {
                            let source_type = source.get_type();
                            let new_count = sources_count.get(&source_type).unwrap_or(&0) + 1;
                            sources_count.insert(source_type, new_count);
                            sources_total_count += 1;
                        }
                    }


                }

            }

            if let Some(flatpak_module) = FlatpakModuleDescription::load_from_file(file_path.to_string()) {

            }

        }

    }

    for (source_type, source_count) in sources_count {
        println!("{}: {} ({}/{})%", source_type, source_count / sources_total_count, source_count, sources_total_count);
    }

    fpm::logger::init();
}
