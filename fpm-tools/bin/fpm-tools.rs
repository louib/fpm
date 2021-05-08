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
        let flathub_repos = match get_flathub_repos() {
            Ok(r) => r,
            Err(e) => panic!(e),
        };
        for flathub_repo in flathub_repos.split('\n') {
            // mine_repository(&mut db, &repo_url);
        }
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

/// Gets all the flathub repositories' URLs, one on each line.
pub fn get_flathub_repos() -> Result<String, String> {
    let flathub_repos_dump_path = format!("{}/flathub.txt", fpm::db::Database::get_repos_db_path());
    let flathub_repos_dump_path = path::Path::new(&flathub_repos_dump_path);

    // Reuse the dump if it exists.
    if flathub_repos_dump_path.is_file() {
        log::info!("Dump of Flathub repos exists, not fetching from GitHub.");
        return match fs::read_to_string(flathub_repos_dump_path) {
            Ok(content) => Ok(content),
            Err(e) => Err(e.to_string()),
        };
    }

    log::info!("Fetching Flathub repos from GitHub.");
    let flathub_repos = fpm_tools::hubs::github::get_org_repos("flathub");
    log::info!("There are {} flathub repos.", flathub_repos.len());

    let mut flathub_repos_dump = "".to_string();
    for flathub_repo in &flathub_repos {
        let repo_url = &flathub_repo.vcs_urls[0];
        // For some reason, the valvesoftware.Steam.CompatibilityTool.Proton
        // project causes an infinite loop when we try to clone it...
        // FIXME this should be handled in the mining phase.
        if repo_url.contains("CompatibilityTool.Proton") {
            continue;
        }

        flathub_repos_dump += &format!("{}\n", repo_url);
    }

    match fs::write(flathub_repos_dump_path, &flathub_repos_dump) {
        Ok(_) => {},
        Err(e) => {
            log::warn!("Could not save the Flathub repos dump to {}: {}.", flathub_repos_dump_path.display(), e);
        },
    };

    Ok(flathub_repos_dump)
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
        if main_module_url.ends_with(".git") && main_module_url.starts_with("https://") && main_module_url != repo_url {
            mine_repository(db, &main_module_url);
        }
        println!("MANIFEST MAX DEPTH {} {}", flatpak_manifest.get_max_depth(), file_path);

        for module in &flatpak_manifest.modules {
            if let FlatpakModule::Description(module_description) = module {
                for url in module_description.get_all_urls() {
                    println!("MODULE URL {}", url);
                }
            }
        }

        for module in flatpak_manifest.modules {
            if let FlatpakModule::Description(module_description) = module {
                db.add_module(module_description);
            }
        }
    }
}
