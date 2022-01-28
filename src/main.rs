//! This is the binary crate for the `fpm` Flatpak module manager.
//! To get the list of available commands, run `fpm -h`.
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path;
use std::process::{exit, Command, Stdio};

// TODO tune built-in attributes
// From https://doc.rust-lang.org/reference/items/modules.html#attributes-on-modules
// The built-in attributes that have meaning on a module are cfg, deprecated, doc,
// the lint check attributes, path, and no_implicit_prelude.
// Modules also accept macro attributes.
#[macro_use]
extern crate clap;

use clap::{AppSettings, App, ArgMatches, Parser, Subcommand};

use flatpak_rs::application::FlatpakApplication;
use flatpak_rs::build_system::FlatpakBuildSystem;
use flatpak_rs::module::{FlatpakModule, FlatpakModuleItem};
use flatpak_rs::source::FlatpakSource;
use fpm_core::project::SoftwareProject;

// This might need to become a regex at some point, to allow fpm to manage multiple module
// manifests at the same time.
const FPM_MODULES_MANIFEST_PATH: &str = "fpm-modules.yaml";
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

mod config;
mod utils;
mod version;

/// CLI tool for managing Flatpak manifests and workspaces
#[derive(Parser)]
#[clap(name = "fpm")]
#[clap(version = env!("CARGO_PKG_VERSION"))]
#[clap(about = "CLI tool for managing Flatpak manifests and workspaces", long_about = None)]
struct Fpm {
    #[clap(subcommand)]
    command: SubCommand,
}

#[derive(Subcommand)]
enum SubCommand {
    /// Formats a Flatpak manifest.
    #[clap(setting(AppSettings::ArgRequiredElseHelp))]
    Search {
        /// The term to search for in the database.
        search_term: String,
    },
}


fn main() {
    fpm_core::logger::init();
    let args = Fpm::parse();
    match &args.command {
        SubCommand::Search { search_term } => {
            if search_term.len() < 3 {
                panic!("{} is too short for a search term!", search_term);
            }

            log::debug!("Searching for {} in the modules.", &search_term);
            let db = fpm_core::db::Database::get_database();
            let modules: Vec<&FlatpakModule> = db.search_modules(search_term);
            for module in modules {
                let main_url = match module.get_main_url() {
                    Some(u) => u,
                    None => continue,
                };
                println!(
                    "{: <22} {: <30} {: <12} {}.",
                    fpm_core::utils::get_module_hash(module),
                    module.name,
                    module.get_buildsystem().unwrap_or("unknown".to_string()),
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
        },
    }

    let yaml = load_yaml!("fpm.yml");
    let mut fpm_app: App = App::from_yaml(yaml).version(APP_VERSION);

    // Here we could use get_matches_safe and override the error messages.
    // See https://docs.rs/clap/2.33.1/clap/struct.App.html#method.get_matches_safe
    let help_text = fpm_app.render_usage().clone();
    let matches: ArgMatches = fpm_app.get_matches();

    if matches.is_present("version") {
        println!("{}", APP_VERSION);
        exit(0);
    }

    // let mut options: HashMap<String, bool> = HashMap::new();
    let mut arguments: HashMap<String, String> = HashMap::new();

    let command_name = match matches.subcommand_name() {
        Some(command_name) => command_name,
        None => {
            eprintln!("Please provide a command to execute.");
            eprintln!("{}", help_text);
            exit(1);
        }
    };

    let subcommand_matches = match matches.subcommand_matches(command_name) {
        Some(subcommand_matches) => subcommand_matches,
        None => {
            eprintln!("Invalid arguments for command {}", command_name);
            eprintln!("{}", help_text);
            exit(1);
        }
    };

    arguments.entry("manifest_file_path".to_string()).or_insert(
        subcommand_matches
            .value_of("manifest_file_path")
            .unwrap_or("")
            .to_string(),
    );
    arguments.entry("search_term".to_string()).or_insert(
        subcommand_matches
            .value_of("search_term")
            .unwrap_or("")
            .to_string(),
    );
    arguments.entry("package_name".to_string()).or_insert(
        subcommand_matches
            .value_of("package_name")
            .unwrap_or("")
            .to_string(),
    );
    arguments
        .entry("env_name".to_string())
        .or_insert(subcommand_matches.value_of("env_name").unwrap_or("").to_string());
    arguments
        .entry("command".to_string())
        .or_insert(subcommand_matches.value_of("command").unwrap_or("").to_string());

    let exit_code = run(command_name, arguments);
    exit(exit_code);
}

pub fn run(command_name: &str, args: HashMap<String, String>) -> i32 {
    log::debug!("running command {}.", command_name);

    let mut config = match crate::config::read_or_init_config() {
        Ok(c) => c,
        Err(e) => panic!("Could not load or init config: {}", e),
    };

    if command_name == "search" {
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
        let modules: Vec<&FlatpakModule> = db.search_modules(package_name);
        let mut module_to_install: Option<FlatpakModule> = None;
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
                .insert(0, FlatpakModuleItem::Description(module));

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

    if command_name == "clean" {
        let flatpak_build_cache_dir = path::Path::new(crate::utils::DEFAULT_FLATPAK_BUILDER_CACHE_DIR);
        if flatpak_build_cache_dir.is_dir() {
            println!("Removing {}.", crate::utils::DEFAULT_FLATPAK_BUILDER_CACHE_DIR);
            fs::remove_dir_all(crate::utils::DEFAULT_FLATPAK_BUILDER_CACHE_DIR).unwrap();
        }

        let flatpak_build_output_dir = path::Path::new(crate::utils::DEFAULT_FLATPAK_BUILDER_OUTPUT_DIR);
        if flatpak_build_output_dir.is_dir() {
            println!("Removing {}.", crate::utils::DEFAULT_FLATPAK_BUILDER_OUTPUT_DIR);
            fs::remove_dir_all(crate::utils::DEFAULT_FLATPAK_BUILDER_OUTPUT_DIR).unwrap();
        }
    }

    if command_name == "ls" {
        let git_cache_dir = path::Path::new(crate::utils::DEFAULT_GIT_CACHE_DIR);
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
            if file_path.contains(crate::utils::DEFAULT_GIT_CACHE_DIR) {
                continue;
            }

            if FlatpakApplication::load_from_file(file_path.to_string()).is_ok() {
                println!("{} (app manifest)", file_path);
                found_manifest = true;
            }

            if FlatpakModule::load_from_file(file_path.to_string()).is_ok() {
                if file_path.ends_with(FPM_MODULES_MANIFEST_PATH) {
                    continue;
                }
                println!("{} (module manifest)", file_path);
            }

            if FlatpakSource::load_from_file(file_path.to_string()).is_ok() {
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
        .arg(crate::utils::DEFAULT_FLATPAK_BUILDER_OUTPUT_DIR)
        .arg(manifest_path)
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let output = match output.wait_with_output() {
        Ok(o) => o,
        Err(e) => return Err(e.to_string()),
    };
    if !output.status.success() {
        return Err("Could not run flatpak build.".to_string());
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
