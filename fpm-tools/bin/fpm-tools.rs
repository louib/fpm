use std::path;
use std::fs;
use std::env;
use std::process::exit;
use std::io::{self, BufRead, Write};

use fpm::flatpak_manifest::{FlatpakManifest, FlatpakModule};

fn main() {
    let mut exit_code = 0;
    fpm::logger::init();

    // TODO might need to use std::env::args_os instead, if
    // the args contain unicode.
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Require 1 argument.");
        exit(1);
    }

    let command_name = &args[1];

    if command_name == &"import-flathub-shared-modules".to_string() {
        let mut modules: Vec<FlatpakModule> = vec![];
        let mut db = fpm::db::Database::get_database();
        let repo_path = match fpm::utils::clone_git_repo(
            &"https://github.com/flathub/shared-modules.git"
        ) {
            Ok(p) => p,
            Err(e) => {
                panic!("Could not glone flathub shared modules repo.");
            }
        };
        let all_repo_paths = match fpm::utils::get_all_paths(path::Path::new(&repo_path)) {
            Ok(p) => p,
            Err(e) => {
                panic!("Could not get paths in flathub shared modules repo.");
            }
        };

        let mut flatpak_modules: Vec<FlatpakModule> = vec![];
        for file_path in &all_repo_paths {
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
            if flatpak_module.sources.len() == 0 {
                continue;
            }

            db.add_module(flatpak_module);
        }

    }

    if command_name == &"import-flathub-manifests".to_string() {
        let mut db = fpm::db::Database::get_database();
        let all_flathub_repos = fpm_tools::hubs::github::get_org_repos("flathub");
        for flathub_repo in &all_flathub_repos {
            let repo_url = &flathub_repo.vcs_urls[0];
            let repo_dir = match fpm::utils::clone_git_repo(&repo_url) {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("Could not clone repo {}: {}", &repo_url, e);
                    continue;
                },
            };
            // TODO we should also rewind on all the commits of that repo?
            let repo_file_paths = match fpm::utils::get_all_paths(path::Path::new(&repo_dir)) {
                Ok(paths) => paths,
                Err(message) => {
                    log::error!("Could not get the file paths for {} :sad: {}", repo_dir, message);
                    continue;
                }
            };
            for file_path in &repo_file_paths {
                let file_path = file_path.to_str().unwrap();
                if file_path.contains(".git/") {
                    continue;
                }

                // We're a bit aggressive here, we could try parsing only the files
                // that match exactly the flatpak path convention.
                if !file_path.ends_with(".json") && !file_path.ends_with(".yaml") && !file_path.ends_with(".yml") {
                    continue;
                }
                let manifest_content = match fs::read_to_string(file_path) {
                    Ok(content) => content,
                    Err(e) => {
                        log::debug!("Could not read manifest file {}: {}.", &file_path, e);
                        continue;
                    }
                };
                let flatpak_manifest = match FlatpakManifest::parse(&manifest_content) {
                    Some(m) => m,
                    None => continue,
                };

                for module in flatpak_manifest.modules {
                    db.add_module(module);
                }

                // TODO infer projects from the modules when possible.
            }

        }
        println!("There are {} flathub repos.", all_flathub_repos.len());
    }

    if command_name == &"import-projects-from-gitlabs".to_string() {
        // There is a list of all the public GitLab instances hosted here
        // https://wiki.p2pfoundation.net/List_of_Community-Hosted_GitLab_Instances
        let mut db = fpm::db::Database::get_database();
        fpm_tools::hubs::gitlab::get_and_add_repos("gitlab.gnome.org", "PB_GNOME_GITLAB_TOKEN", &mut db);
        fpm_tools::hubs::gitlab::get_and_add_repos("source.puri.sm", "PB_PURISM_GITLAB_TOKEN", &mut db);
        fpm_tools::hubs::gitlab::get_and_add_repos("salsa.debian.org", "PB_DEBIAN_GITLAB_TOKEN", &mut db);
        // KDE was recently migrated to GitLab.
        // See https://gitlab.com/gitlab-org/gitlab-foss/-/issues/53206 for details.
        fpm_tools::hubs::gitlab::get_and_add_repos("invent.kde.org", "PB_KDE_GITLAB_TOKEN", &mut db);
        fpm_tools::hubs::gitlab::get_and_add_repos("code.videolan.org", "PB_VLC_GITLAB_TOKEN", &mut db);
        fpm_tools::hubs::gitlab::get_and_add_repos("gitlab.haskell.org", "PB_HASKELL_GITLAB_TOKEN", &mut db);
        fpm_tools::hubs::gitlab::get_and_add_repos("devel.trisquel.info", "PB_TRISQUEL_GITLAB_TOKEN", &mut db);
        fpm_tools::hubs::gitlab::get_and_add_repos("gitlab.freedesktop.org", "PB_XDG_GITLAB_TOKEN", &mut db);
    }

    if command_name == &"import-projects-from-gitlab-com".to_string() {
        let mut db = fpm::db::Database::get_database();
        fpm_tools::hubs::gitlab::get_and_add_repos("gitlab.com", "PB_GITLAB_TOKEN", &mut db);
    }

    if command_name == &"import-projects-from-github-com".to_string() {
        let mut db = fpm::db::Database::get_database();
        fpm_tools::hubs::github::get_and_add_repos(&mut db);
    }

    if command_name == &"import-brew-recipes".to_string() {
        let mut db = fpm::db::Database::get_database();
        fpm_tools::hubs::brew::get_and_add_recipes(&mut db);
    }

    exit(exit_code);
}
