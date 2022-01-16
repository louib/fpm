use std::collections::HashMap;
use std::env;
use std::process::{Command, Stdio};

pub mod utils;
pub mod vcpkg_manifest;

mod config;
mod version;

use flatpak_rs::application::FlatpakApplication;
use flatpak_rs::module::{FlatpakModule, FlatpakModuleDescription};
use flatpak_rs::source::FlatpakSourceDescription;
use fpm_core::project::SoftwareProject;

use std::fs;
use std::path;

const DEFAULT_GIT_CACHE_DIR: &str = ".git/";
// This might need to become a regex at some point, to allow fpm to manage multiple module
// manifests at the same time.
const FPM_MODULES_MANIFEST_PATH: &str = "fpm-modules.yaml";
const DEFAULT_PACKAGE_LIST_SEP: &str = ",";

pub fn run(command_name: &str, args: HashMap<String, String>) -> i32 {
    fpm_core::logger::init();

    log::debug!("running command {}.", command_name);

    let mut config = match crate::config::read_or_init_config() {
        Ok(c) => c,
        Err(e) => panic!("Could not load or init config: {}", e),
    };

    if command_name == "lint" {
        let manifest_file_path = args
            .get("manifest_file_path")
            .expect("an input file is required!");

        let flatpak_manifest = match FlatpakApplication::load_from_file(manifest_file_path.to_string()) {
            Ok(m) => m,
            Err(e) => {
                eprintln!("Could not parse manifest file at {}: {}.", manifest_file_path, e);
                return 1;
            }
        };

        let manifest_dump = match flatpak_manifest.dump() {
            Ok(d) => d,
            Err(_e) => return 1,
        };

        match fs::write(path::Path::new(manifest_file_path), manifest_dump) {
            Ok(content) => content,
            Err(e) => {
                eprintln!("could not write file {}: {}.", manifest_file_path, e);
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

        let flatpak_manifest = match FlatpakApplication::load_from_file(manifest_file_path.to_string()) {
            Ok(m) => m,
            Err(e) => {
                eprintln!("Could not parse manifest file at {}: {}.", manifest_file_path, e);
                return 1;
            }
        };

        let mut separator = DEFAULT_PACKAGE_LIST_SEP;
        if args.contains_key("separator") {
            separator = args.get("separator").unwrap();
        }

        let mut output: String = String::from("");
        for module in &flatpak_manifest.get_all_modules_recursively() {
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

        log::debug!("Searching for {} in the modules.", &search_term);
        let db = fpm_core::db::Database::get_database();
        let modules: Vec<&FlatpakModuleDescription> = db.search_modules(search_term);
        for module in modules {
            let main_url = match module.get_main_url() {
                Some(u) => u,
                None => continue,
            };
            println!(
                "{: <22} {: <30} {: <12} {}.",
                fpm_core::utils::get_module_hash(module),
                module.name,
                module.buildsystem.to_string(),
                main_url
            );
        }

        log::debug!("Searching for {} in the projects.", &search_term);
        let projects: Vec<&SoftwareProject> = db.search_projects(search_term);
        for project in projects {
            println!(
                "found candidate project {} ({}).",
                project.name,
                project.get_main_vcs_url()
            );
        }
    }

    if command_name == "install" {
        let package_name = match args.get("package_name") {
            Some(n) => n,
            None => {
                eprintln!("a package name to install is required!");
                return 1;
            }
        };

        if package_name.len() < 4 {
            eprintln!("Module name is too short");
            return 1;
        }

        let db = fpm_core::db::Database::get_database();
        let modules: Vec<&FlatpakModuleDescription> = db.search_modules(package_name);
        let mut module_to_install: Option<FlatpakModuleDescription> = None;
        for module in modules {
            println!("{}", module.dump().unwrap());
            let answer =
                crate::utils::ask_yes_no_question("Is this the module you want to install".to_string());
            if answer {
                module_to_install = Some(module.clone());
                break;
            }
        }

        if let Some(module) = module_to_install {
            let manifest_path = match get_manifest_file_path(args.get("manifest_file_path")) {
                Some(m) => m,
                None => {
                    return 1;
                }
            };
            log::info!("Using Flatpak manifest at {}", manifest_path);

            let mut flatpak_manifest = match FlatpakApplication::load_from_file(manifest_path.to_string()) {
                Ok(m) => m,
                Err(e) => {
                    log::error!("Could not parse Flatpak manifest at {}: {}", &manifest_path, e);
                    return 1;
                }
            };

            flatpak_manifest
                .modules
                .insert(0, flatpak_rs::module::FlatpakModule::Description(module));

            let manifest_dump = match flatpak_manifest.dump() {
                Ok(d) => d,
                Err(_e) => return 1,
            };

            match fs::write(path::Path::new(&manifest_path), manifest_dump) {
                Ok(content) => content,
                Err(e) => {
                    eprintln!("could not write file {}: {}.", manifest_path, e);
                    return 1;
                }
            };
        }
    }

    if command_name == "parse" {
        let manifest_file_path = match args.get("manifest_file_path") {
            Some(p) => p,
            None => {
                eprintln!("a manifest file is required for this command!");
                return 1;
            }
        };

        if let Err(e) =
            flatpak_rs::application::FlatpakApplication::load_from_file(manifest_file_path.to_string())
        {
            println!("Could not parse manifest file: {}", e);
            return 1;
        }

        println!("Parsed manifest file at {}.", manifest_file_path);
        return 0;
    }

    if command_name == "make" {
        let manifest_path = match get_manifest_file_path(args.get("manifest_file_path")) {
            Some(m) => m,
            None => {
                return 1;
            }
        };
        log::info!("Using Flatpak manifest at {}", manifest_path);

        if let Err(e) = FlatpakApplication::load_from_file(manifest_path.to_string()) {
            log::error!("Could not parse Flatpak manifest at {}: {}", &manifest_path, e);
            return 1;
        }

        match run_build(&manifest_path) {
            Ok(_) => return 0,
            Err(_) => return 1,
        };
    }

    if command_name == "run" {}

    if command_name == "clean" {}

    if command_name == "ls" {
        let git_cache_dir = path::Path::new(DEFAULT_GIT_CACHE_DIR);
        if !git_cache_dir.is_dir() {
            eprintln!("This does not seem like a git project (.git/ was not found).");
            return 1;
        }

        // FIXME only enable with a `-a` option.
        let list_all = true;

        let mut found_manifest = false;
        let file_paths = match fpm_core::utils::get_all_paths(path::Path::new("./")) {
            Ok(paths) => paths,
            Err(message) => {
                eprintln!("Could not get the file paths :sad: {}", message);
                return 1;
            }
        };
        // TODO print also those already matched to workspaces.
        for file_path in file_paths.iter() {
            if !file_path.is_file() {
                continue;
            }

            let file_path = file_path.to_str().unwrap();
            if file_path.contains(DEFAULT_GIT_CACHE_DIR) {
                continue;
            }

            if FlatpakApplication::load_from_file(file_path.to_string()).is_ok() {
                println!("{} (app manifest)", file_path);
                found_manifest = true;
            }

            if FlatpakModuleDescription::load_from_file(file_path.to_string()).is_ok() {
                if file_path.ends_with(FPM_MODULES_MANIFEST_PATH) {
                    continue;
                }
                println!("{} (module manifest)", file_path);
            }

            if FlatpakSourceDescription::load_from_file(file_path.to_string()).is_ok() {
                println!("{} (sources manifest)", file_path);
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

    if command_name == "stats" {
        let db = fpm_core::db::Database::get_database();
        println!("{}", db.get_stats());
        return 0;
    }

    log::debug!("Finishing...");
    return 0;
}

fn run_build(manifest_path: &str) -> Result<(), String> {
    let output = Command::new("flatpak-builder")
        .arg("--user")
        .arg("--force-clean")
        .arg("build/")
        .arg(manifest_path)
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let output = match output.wait_with_output() {
        Ok(o) => o,
        Err(e) => return Err(e.to_string()),
    };
    if !output.status.success() {
        return Err("Could not checkout git ref.".to_string());
    }
    Ok(())
}

pub fn get_manifest_file_path(path_arg: Option<&String>) -> Option<String> {
    if let Some(manifest_file_path) = path_arg {
        if manifest_file_path.trim().len() != 0 {
            return Some(manifest_file_path.to_string());
        }
    };

    let manifest_path = match crate::config::get_manifest_path() {
        Ok(m) => return Some(m),
        Err(e) => {
            let current_dir = env::current_dir().unwrap();
            match crate::utils::get_candidate_flatpak_manifests(current_dir.to_str().unwrap()) {
                Ok(candidate_manifests) => {
                    if candidate_manifests.len() != 1 {
                        log::error!("Found {} candidate Flatpak manifests.", candidate_manifests.len());
                        return None;
                    }
                    candidate_manifests[0].clone()
                }
                Err(e) => {
                    log::error!("Could not find candidate Flatpak manifests: {}.", e);
                    return None;
                }
            }
        }
    };
    return Some(manifest_path);
}
