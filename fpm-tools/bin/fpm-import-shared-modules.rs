use std::path;
use std::fs;
use std::env;
use std::process::exit;
use std::io::{self, BufRead, Write};

use fpm::flatpak_manifest::{FlatpakManifest, FlatpakModule};


fn main() {
    fpm::logger::init();

    // TODO might need to use std::env::args_os instead, if
    // the args contain unicode.
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        panic!("Requires 1 argument: the list of source to import from, or `all` for all the sources.");
    }

    let sources = &args[1];

    if sources.contains("github-flathub-shared-modules") {
        let mut modules: Vec<FlatpakModule> = vec![];
        let mut db = fpm::db::Database::get_database();
        let repo_path = match fpm::utils::clone_git_repo(
            &"https://github.com/flathub/shared-modules.git"
        ) {
            Ok(p) => p,
            Err(e) => {
                panic!("Could not clone flathub shared modules repo.");
            }
        };
        let all_paths_in_repo = match fpm::utils::get_all_paths(path::Path::new(&repo_path)) {
            Ok(p) => p,
            Err(e) => {
                panic!("Could not get paths in flathub shared modules repo.");
            }
        };

        let mut flatpak_modules: Vec<FlatpakModule> = vec![];
        for file_path in &all_paths_in_repo {
            let file_path_str = file_path.to_str().unwrap();

            let file_content = match fs::read_to_string(file_path) {
                Ok(content) => content,
                Err(e) => {
                    log::debug!("Could not read file {}: {}.", file_path_str, e);
                    continue;
                }
            };

            log::debug!("Trying to parse Flatpak module at {}.", file_path_str);
            let module: FlatpakModule = match serde_json::from_str(&file_content) {
                Ok(m) => m,
                Err(e) => {
                    log::debug!("Could not parse file {}: {}.", file_path_str, e);
                    continue;
                }
            };

            println!("Parsed Flatpak module at {}.", file_path_str);
            flatpak_modules.push(module);
        }

        println!("Importing {} Flatpak module.", &flatpak_modules.len());
        for flatpak_module in flatpak_modules {
            if let FlatpakModule::Description(module_description) = flatpak_module {
                if module_description.sources.len() == 0 {
                    continue;
                }

                db.add_module(module_description);
            }
        }

    }

    exit(0);
}
