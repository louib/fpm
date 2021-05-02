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

    if command_name == &"import-flathub-manifests".to_string() {
        let mut db = fpm::db::Database::get_database();
        let all_flathub_repos = fpm_tools::hubs::github::get_org_repos("flathub");
        for flathub_repo in &all_flathub_repos {
            let repo_url = &flathub_repo.vcs_urls[0];
            // FIXME for some reason, the valvesoftware.Steam.CompatibilityTool.Proton
            // project causes an infinite loop when we try to clone it...
            if repo_url.contains("CompatibilityTool.Proton") {
                continue;
            }

            mine_repository(&mut db, &repo_url);
        }
        println!("There are {} flathub repos.", all_flathub_repos.len());
    }

    if command_name == &"import-projects-from-gitlabs".to_string() {
        // There is a list of all the public GitLab instances hosted here
        // https://wiki.p2pfoundation.net/List_of_Community-Hosted_GitLab_Instances
        let mut db = fpm::db::Database::get_database();
        fpm_tools::hubs::gitlab::get_and_add_repos("gitlab.gnome.org", "FPM_GNOME_GITLAB_TOKEN", &mut db);
        fpm_tools::hubs::gitlab::get_and_add_repos("source.puri.sm", "FPM_PURISM_GITLAB_TOKEN", &mut db);
        fpm_tools::hubs::gitlab::get_and_add_repos("salsa.debian.org", "FPM_DEBIAN_GITLAB_TOKEN", &mut db);
        // KDE was recently migrated to GitLab.
        // See https://gitlab.com/gitlab-org/gitlab-foss/-/issues/53206 for details.
        fpm_tools::hubs::gitlab::get_and_add_repos("invent.kde.org", "FPM_KDE_GITLAB_TOKEN", &mut db);
        fpm_tools::hubs::gitlab::get_and_add_repos("code.videolan.org", "FPM_VLC_GITLAB_TOKEN", &mut db);
        fpm_tools::hubs::gitlab::get_and_add_repos("gitlab.haskell.org", "FPM_HASKELL_GITLAB_TOKEN", &mut db);
        fpm_tools::hubs::gitlab::get_and_add_repos("devel.trisquel.info", "FPM_TRISQUEL_GITLAB_TOKEN", &mut db);
        fpm_tools::hubs::gitlab::get_and_add_repos("gitlab.freedesktop.org", "FPM_XDG_GITLAB_TOKEN", &mut db);
    }

    if command_name == &"import-projects-from-gitlab-com".to_string() {
        let mut db = fpm::db::Database::get_database();
        fpm_tools::hubs::gitlab::get_and_add_repos("gitlab.com", "FPM_GITLAB_TOKEN", &mut db);
    }

    if command_name == &"import-projects-from-github-com".to_string() {
        let mut db = fpm::db::Database::get_database();
        fpm_tools::hubs::github::get_and_add_repos(&mut db);
    }

    if command_name == &"import-brew-recipes".to_string() {
        let mut db = fpm::db::Database::get_database();
        fpm_tools::hubs::brew::get_and_add_recipes(&mut db);
    }

    if command_name == &"extract-projects-from-modules" {
        // TODO infer projects from the modules when possible.
    }

    exit(exit_code);
}

pub fn mine_repository(db: &mut fpm::db::Database, repo_url: &str) {
    let repo_dir = match fpm::utils::clone_git_repo(&repo_url) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Could not clone repo {}: {}", &repo_url, e);
            return;
        },
    };

    // TODO we should also rewind on all the commits of that repo?
    let repo_file_paths = match fpm::utils::get_all_paths(path::Path::new(&repo_dir)) {
        Ok(paths) => paths,
        Err(message) => {
            log::error!("Could not get the file paths for {} :sad: {}", repo_dir, message);
            return;
        }
    };
    for file_path in &repo_file_paths {
        let file_path = file_path.to_str().unwrap();
        if file_path.contains(".git/") {
            continue;
        }
        // We handle the shared modules separately.
        if file_path.contains(".shared-modules/") {
            continue;
        }

        let flatpak_manifest = match FlatpakManifest::load_from_file(file_path.to_string()) {
            Some(m) => m,
            None => continue,
        };

        let main_module_url = flatpak_manifest.get_main_module_url();
        let main_module_url = match main_module_url {
            Some(u) => u,
            None => String::from(""),
        };
        if main_module_url.ends_with(".git") && main_module_url != repo_url {
            println!("ALSO MINING A MAIN MODULE GIT URL {}", main_module_url);
            mine_repository(db, &main_module_url);
        }

        for module in flatpak_manifest.modules {
            if let FlatpakModule::Description(module_description) = module {
                db.add_module(module_description);
            }
        }
    }
}
