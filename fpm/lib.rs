use std::collections::HashMap;

pub mod db;
pub mod flatpak_manifest;
pub mod logger;
pub mod projects;
pub mod utils;

mod config;
mod version;

pub use flatpak_manifest::{FlatpakModule, FlatpakModuleDescription};
pub use projects::SoftwareProject;

use std::fs;
use std::path;

const DEFAULT_GIT_CACHE_DIR: &str = ".git/";
const DEFAULT_PACKAGE_LIST_SEP: &str = ",";

pub fn run(command_name: &str, args: HashMap<String, String>) -> i32 {
    logger::init();

    log::debug!("running command {}.", command_name);

    let mut config = match crate::config::read_or_init_config() {
        Ok(c) => c,
        Err(e) => panic!("Could not load or init config: {}", e),
    };

    if command_name == "lint" {
        let manifest_file_path = args
            .get("manifest_file_path")
            .expect("an input file is required!");

        let mut flatpak_manifest =
            match crate::flatpak_manifest::FlatpakManifest::load_from_file(manifest_file_path.to_string()) {
                Some(m) => m,
                None => return 1,
            };

        let manifest_dump = match flatpak_manifest.dump() {
            Ok(d) => d,
            Err(_e) => return 1,
        };

        match fs::write(path::Path::new(manifest_file_path), manifest_dump) {
            Ok(content) => content,
            Err(e) => {
                eprintln!("could not write file {}.", manifest_file_path);
                return 1;
            }
        };

        eprintln!("Dumped the manifest!");
        return 0;
    }

    if command_name == "get-package-list" {
        let manifest_file_path = args
            .get("manifest_file_path")
            .expect("a manifest file is required!");

        let flatpak_manifest =
            match crate::flatpak_manifest::FlatpakManifest::load_from_file(manifest_file_path.to_string()) {
                Some(m) => m,
                None => return 1,
            };

        let mut separator = DEFAULT_PACKAGE_LIST_SEP;
        if args.contains_key("separator") {
            separator = args.get("separator").unwrap();
        }

        let mut output: String = String::from("");
        // TODO this should get all the modules recursively.
        for module in &flatpak_manifest.modules {
            if !output.is_empty() {
                output.push_str(&separator)
            }
            if let FlatpakModule::Description(module_description) = module {
                output.push_str(&module_description.name);
            }
        }
        println!("{}", output);
    }

    if command_name == "search" {
        let search_term = match args.get("search_term") {
            Some(search_term) => search_term,
            None => {
                eprintln!("A search term is required!");
                return 1;
            }
        };
        if search_term.len() < 3 {
            eprintln!("{} is too short for a search term!", search_term);
            return 1;
        }
        eprintln!("Search for {} in the projects database.", &search_term);

        let db = crate::db::Database::get_database();
        let modules: Vec<&FlatpakModuleDescription> = db.search_modules(search_term);
        for module in modules {
            println!(
                "found candidate module {} ({}).",
                module.name,
                module.get_main_url().unwrap()
            );
        }
        let projects: Vec<&SoftwareProject> = db.search_projects(search_term);
        for project in projects {
            println!(
                "found candidate project {} ({}).",
                project.name,
                project.get_main_vcs_url()
            );
        }
    }

    if command_name == "install" {}

    if command_name == "make" {
        let candidate_flatpak_manifests = match crate::utils::get_candidate_flatpak_manifests("./") {
            Ok(m) => m,
            Err(e) => {
                log::error!("Error while search for Flatpak manifests: {}.", e);
                return 1;
            },
        };

        if candidate_flatpak_manifests.len() == 0 {
            log::error!("Could not find any Flatpak manifest to build with.");
            return 1;
        }

        if candidate_flatpak_manifests.len() != 1 {
            log::error!("Too many Flatpak manifests to pick from. Use workspaces.");
            return 1;
        }


        // make without argument runs the only manifest if there is only one
    }

    if command_name == "run" {}

    if command_name == "ls" {
        let git_cache_dir = path::Path::new(DEFAULT_GIT_CACHE_DIR);
        if !git_cache_dir.is_dir() {
            eprintln!("This does not seem like a git project (.git/ was not found).");
            return 1;
        }

        let mut found_manifest = false;
        let file_paths = match utils::get_all_paths(path::Path::new("./")) {
            Ok(paths) => paths,
            Err(message) => {
                eprintln!("Could not get the file paths :sad: {}", message);
                return 1;
            }
        };
        // TODO print also those already matched to workspaces.
        for path in file_paths.iter() {
            let file_path = path;
            let file_path_str = file_path.to_str().unwrap();
            if !file_path.is_file() {
                continue;
            }
            // TODO Test that if it starts with the cache directories listed above,
            // you skip the file.

            if crate::flatpak_manifest::FlatpakManifest::load_from_file(file_path_str.to_string()).is_some() {
                println!("{}", file_path_str);
                found_manifest = true;
            }
        }

        if !found_manifest {
            eprintln!("No available workspace found for the project. Try running `ls -p`.");
        } else {
            println!("Use `checkout` to select a workspace.");
        }
    }

    if command_name == "checkout" {
        let env_name = match args.get("env_name") {
            Some(n) => n,
            None => panic!("An env name is required to checkout."),
        };

        if let Some(current_workspace) = &config.current_workspace {
            if current_workspace == env_name {
                println!("Already in workspace {}.", env_name);
                return 0;
            }
        }

        if !config.workspaces.contains_key(env_name) {
            eprintln!(
                "Workspace {} does not exist. Use `ls` to list the available workspaces and manifests.",
                env_name
            );
            return 1;
        }

        config.current_workspace = Some(env_name.to_string());
        match crate::config::write_config(&config) {
            Ok(c) => c,
            Err(e) => panic!("Could not write config: {}", e),
        };
    }

    if command_name == "create" {
        let env_name = match args.get("env_name") {
            Some(n) => n,
            None => panic!("An env name is required to checkout."),
        };

        if let Some(current_workspace) = &config.current_workspace {
            if current_workspace == env_name {
                println!("Already in workspace {}.", env_name);
                return 0;
            }
        }

        if config.workspaces.contains_key(env_name) {
            eprintln!("Workspace {} already exists.", env_name);
            return 1;
        }

        let manifest_file_path = match args.get("manifest_file_path") {
            Some(p) => p,
            None => {
                eprintln!("a manifest file is required to create a new workspace!");
                // TODO handle reading from stdin.
                return 1;
            }
        };

        config
            .workspaces
            .insert(env_name.to_string(), manifest_file_path.to_string());
        config.current_workspace = Some(env_name.to_string());
        match crate::config::write_config(&config) {
            Ok(c) => c,
            Err(e) => panic!("Could not write config: {}", e),
        };
        println!(
            "🗃 Created workspace {} with manifest file {}.",
            env_name, manifest_file_path
        );
    }

    if command_name == "status" {
        let current_workspace = match config.current_workspace {
            Some(workspace) => workspace,
            None => "".to_string(),
        };

        if current_workspace.len() == 0 {
            println!("Not in a workspace. Call `ls` to list the workspaces and manifest files.");
            return 0;
        }

        if !config.workspaces.contains_key(&current_workspace) {
            panic!("Workspace {} not found in config!.", current_workspace);
        }

        let manifest_file_path = config.workspaces.get(&current_workspace).unwrap();
        println!("Workspace {} using {}.", current_workspace, manifest_file_path);
    }

    log::debug!("Finishing...");
    return 0;
}
