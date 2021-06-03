use std::collections::HashSet;
use std::path;
use std::fs;
use std::env;
use std::process::exit;

use fpm::flatpak_manifest::{FlatpakManifest, FlatpakModule, FlatpakModuleDescription};

fn main() {
    fpm::logger::init();

    // TODO might need to use std::env::args_os instead, if
    // the args contain unicode.
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        panic!("Requires 1 argument: the list of sources to import from, or `all` for all the sources.");
    }

    let sources = &args[1];

    let mut repos_urls: String = "".to_string();

    if sources.contains("github-flathub-org") {
        repos_urls += &match get_flathub_repos() {
            Ok(r) => r,
            Err(e) => panic!(e),
        };
    }

    if sources.contains("gnome-gitlab-instance") {
        repos_urls += &match get_gitlab_repos("gitlab.gnome.org", "FPM_GNOME_GITLAB_TOKEN") {
            Ok(r) => r,
            Err(e) => panic!(e),
        };
    }

    if sources.contains("purism-gitlab-instance") {
        repos_urls += &match get_gitlab_repos("source.puri.sm", "FPM_PURISM_GITLAB_TOKEN") {
            Ok(r) => r,
            Err(e) => panic!(e),
        };
    }

    if sources.contains("debian-gitlab-instance") {
        repos_urls += &match get_gitlab_repos("salsa.debian.org", "FPM_DEBIAN_GITLAB_TOKEN") {
            Ok(r) => r,
            Err(e) => panic!(e),
        };
    }

    if sources.contains("xdg-gitlab-instance") {
        repos_urls += &match get_gitlab_repos("gitlab.freedesktop.org", "FPM_XDG_GITLAB_TOKEN") {
            Ok(r) => r,
            Err(e) => panic!(e),
        };
    }

    if sources.contains("kde-gitlab-instance") {
        repos_urls += &match get_gitlab_repos("invent.kde.org", "FPM_KDE_GITLAB_TOKEN") {
            Ok(r) => r,
            Err(e) => panic!(e),
        };
    }

    // TODO also get code.videolan.org ??
    // TODO also get gitlab.haskell.org ??
    // TODO also get devel.trisquel.info ??

    if sources.contains("gitlab-search-flatpak") {
        repos_urls += &match search_gitlab("flatpak") {
            Ok(r) => r,
            Err(e) => panic!(e),
        };
    }

    if sources.contains("gitlab-search-flathub") {
        repos_urls += &match search_gitlab("flathub") {
            Ok(r) => r,
            Err(e) => panic!(e),
        };
    }

    if sources.contains("github-search-flatpak") {
        repos_urls += &match search_github("flatpak") {
            Ok(r) => r,
            Err(e) => panic!(e),
        };
    }

    if sources.contains("github-search-flathub") {
        repos_urls += &match search_github("flathub") {
            Ok(r) => r,
            Err(e) => panic!(e),
        };
    }

    if sources.contains("gitlab-com") {
        fpm_tools::hubs::gitlab::get_all_repos("gitlab.com", "FPM_GITLAB_TOKEN");
    }

    if sources.contains("brew-recipes") {
        let mut db = fpm::db::Database::get_database();
        fpm_tools::hubs::brew::get_and_add_recipes(&mut db);
    }

    let db = fpm::db::Database::get_database();
    let mut mined_repos: HashSet<String> = HashSet::new();

    mine_repositories(repos_urls.split('\n').collect(), db, &mut mined_repos);

    exit(0);
}

pub fn mine_repositories(repos_urls: Vec<&str>, mut db: fpm::db::Database, mined_repos: &mut HashSet<String>) {
    let mut next_repos_urls_to_mine: Vec<String> = vec![];

    // Marking all the repos for this discovery round as mined, so that
    // we don't add them for discovery in the next round.
    for repo_url in &repos_urls {
        let repo_url = repo_url.to_string();
        if mined_repos.contains(&repo_url) {
            log::info!("Repo {} was already mined", &repo_url);
            continue;
        }
        mined_repos.insert(repo_url);
    }

    for repo_url in repos_urls {
        if repo_url.trim().is_empty() {
            continue;
        }

        // Found when searching for `flathub` on GitHub.com
        // Too big to be processed.
        if repo_url.contains("fastrizwaan/winepak") {
            continue;
        }

        // Found when searching for `flatpak` on GitHub.com
        // Too big to be processed.
        if repo_url.contains("/ostree") {
            continue;
        }

        // Found on Gnome's GitLab instance
        // Too big to be processed.
        if repo_url.contains("kefqse/origin") {
            continue;
        }

        eprintln!("repo url is {}", repo_url);
        let mined_repos_urls = mine_repository(&mut db, &repo_url);

        for mined_repo_url in mined_repos_urls {
            if mined_repos.contains(&mined_repo_url) {
                continue;
            }
            next_repos_urls_to_mine.push(mined_repo_url);
        }
    }

    if !next_repos_urls_to_mine.is_empty() {
        log::warn!("There are {} other repositories to mine!!!", next_repos_urls_to_mine.len());
        // TODO find a one-liner for that.
        let mut next_repos_urls_to_mine_str = Vec::<&str>::new();
        for url in &next_repos_urls_to_mine {
            next_repos_urls_to_mine_str.push(url);
        }

        mine_repositories(next_repos_urls_to_mine_str, db, mined_repos);
    }

}

/// Search for flatpak and flathub related repos on gitlab.com and
/// return their URLs, one on each line.
pub fn search_gitlab(search_term: &str) -> Result<String, String> {
    let gitlab_repos_search_dump_path = format!("{}/gitlab_repo_search_{}.txt", fpm::db::Database::get_repos_db_path(), search_term);
    let gitlab_repos_search_dump_path = path::Path::new(&gitlab_repos_search_dump_path);

    // Reuse the dump if it exists.
    if gitlab_repos_search_dump_path.is_file() {
        log::info!("Dump of the GitLab search for `{}` exists, not fetching.", &search_term);
        return match fs::read_to_string(gitlab_repos_search_dump_path) {
            Ok(content) => Ok(content),
            Err(e) => Err(e.to_string()),
        };
    }

    log::info!("Searching for {} on GitLab.", &search_term);
    let github_repos = fpm_tools::hubs::gitlab::search_repos(&search_term);
    log::info!("Search for {} returned {} repos.", &search_term, github_repos.len());

    let mut gitlab_repos_search_dump = "".to_string();
    for github_repo in &github_repos {
        let repo_url = &github_repo.http_url_to_repo;
        gitlab_repos_search_dump += &format!("{}\n", repo_url);
    }

    if !gitlab_repos_search_dump.is_empty() {
        match fs::write(gitlab_repos_search_dump_path, &gitlab_repos_search_dump) {
            Ok(_) => {},
            Err(e) => {
                log::warn!("Could not save the dump for GitLab search to {}: {}.", gitlab_repos_search_dump_path.display(), e);
            },
        };
    }

    Ok(gitlab_repos_search_dump)
}

/// Search for flatpak and flathub related repos on github.com and
/// return their URLs, one on each line.
pub fn search_github(search_term: &str) -> Result<String, String> {
    // TODO clean up the search term.
    let github_repos_search_dump_path = format!("{}/github_repo_search_{}.txt", fpm::db::Database::get_repos_db_path(), search_term);
    let github_repos_search_dump_path = path::Path::new(&github_repos_search_dump_path);

    // Reuse the dump if it exists.
    if github_repos_search_dump_path.is_file() {
        log::info!("Dump of the GitHub search for `{}` exists, not fetching.", &search_term);
        return match fs::read_to_string(github_repos_search_dump_path) {
            Ok(content) => Ok(content),
            Err(e) => Err(e.to_string()),
        };
    }

    log::info!("Searching for {} on GitHub.", &search_term);
    let github_repos = fpm_tools::hubs::github::search_repos(&search_term);
    log::info!("Search for {} returned {} repos.", &search_term, github_repos.len());

    let mut github_repos_search_dump = "".to_string();
    for github_repo in &github_repos {
        let repo_url = github_repo.get_git_url();
        github_repos_search_dump += &format!("{}\n", repo_url);
    }

    if !github_repos_search_dump.is_empty() {
        match fs::write(github_repos_search_dump_path, &github_repos_search_dump) {
            Ok(_) => {},
            Err(e) => {
                log::warn!("Could not save the dump for GitHub search to {}: {}.", github_repos_search_dump_path.display(), e);
            },
        };
    }

    Ok(github_repos_search_dump)
}

/// Gets all the repositories' URLs for a specific GitLab instance, one on each line.
pub fn get_gitlab_repos(gitlab_instance_url: &str, gitlab_instance_auth_token_name: &str) -> Result<String, String> {
    let gitlab_instance_dump_key = gitlab_instance_url.replace('.', "_");

    let gitlab_instance_repos_dump_path = format!("{}/{}.txt", fpm::db::Database::get_repos_db_path(), gitlab_instance_dump_key);
    let gitlab_instance_repos_dump_path = path::Path::new(&gitlab_instance_repos_dump_path);

    // Reuse the dump if it exists.
    if gitlab_instance_repos_dump_path.is_file() {
        log::info!("Dump of the repos at GitLab instance {} exists, not fetching.", &gitlab_instance_url);
        return match fs::read_to_string(gitlab_instance_repos_dump_path) {
            Ok(content) => Ok(content),
            Err(e) => Err(e.to_string()),
        };
    }

    log::info!("Fetching repos from GitLab at {}.", &gitlab_instance_url);
    let gitlab_repos = fpm_tools::hubs::gitlab::get_all_repos(&gitlab_instance_url, &gitlab_instance_auth_token_name);
    log::info!("There are {} GitLab repos at {}.", gitlab_repos.len(), &gitlab_instance_url);

    let mut gitlab_repos_dump = "".to_string();
    for gitlab_repo in &gitlab_repos {
        let repo_url = &gitlab_repo.http_url_to_repo;
        gitlab_repos_dump += &format!("{}\n", repo_url);
    }

    if !gitlab_repos_dump.is_empty() {
        match fs::write(gitlab_instance_repos_dump_path, &gitlab_repos_dump) {
            Ok(_) => {},
            Err(e) => {
                log::warn!("Could not save the Flathub repos dump to {}: {}.", gitlab_instance_repos_dump_path.display(), e);
            },
        };
    }

    Ok(gitlab_repos_dump)
}

/// Gets all the repositories' URLs for the Flathub organization hosted
/// on github.com, one on each line.
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
        let repo_url = &flathub_repo.get_git_url();
        // For some reason, the valvesoftware.Steam.CompatibilityTool.Proton
        // project causes an infinite loop when we try to clone it...
        // FIXME this should be handled in the mining phase.
        if repo_url.contains("CompatibilityTool.Proton") {
            continue;
        }

        flathub_repos_dump += &format!("{}\n", repo_url);
    }

    if !flathub_repos_dump.is_empty() {
        match fs::write(flathub_repos_dump_path, &flathub_repos_dump) {
            Ok(_) => {},
            Err(e) => {
                log::warn!("Could not save the Flathub repos dump to {}: {}.", flathub_repos_dump_path.display(), e);
            },
        };
    }

    Ok(flathub_repos_dump)
}

pub fn mine_repository(db: &mut fpm::db::Database, repo_url: &str) -> Vec<String> {
    let mut software_project = fpm::projects::SoftwareProject::default();
    software_project.id = fpm::utils::repo_url_to_reverse_dns(repo_url);
    software_project.vcs_urls.insert(repo_url.to_string());
    // TODO get the root hashes.

    let mut mined_repos_urls: Vec<String> = vec![];
    let mut repo_manifest_count = 0;
    let repo_dir = match fpm::utils::clone_git_repo(&repo_url) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Could not clone repo {}: {}", &repo_url, e);
            return mined_repos_urls;
        },
    };

    // TODO we should also rewind on all the commits of that repo?
    let repo_file_paths = match fpm::utils::get_all_paths(path::Path::new(&repo_dir)) {
        Ok(paths) => paths,
        Err(message) => {
            log::error!("Could not get the file paths for {} :sad: {}", repo_dir, message);
            return mined_repos_urls;
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
        // We handle the shared modules separately.
        if file_path.contains(".shared-modules/") {
            continue;
        }

        if let Some(flatpak_manifest) = FlatpakManifest::load_from_file(file_path.to_string()) {
            let flatpak_manifest_path = file_path.replace(&repo_dir, "");
            software_project.flatpak_app_manifests.insert(flatpak_manifest_path);

            repo_manifest_count += 1;
            log::info!("Parsed a Flatpak manifest at {}", file_path.to_string());

            for module in &flatpak_manifest.modules {
                for url in module.get_all_repos_urls() {
                    println!("MODULE URL {}", url);
                    if url.ends_with(".git") && url.starts_with("https://") {
                        mined_repos_urls.push(url);
                    }
                }
            }

            for module in flatpak_manifest.modules {
                if let FlatpakModule::Description(module_description) = module {
                    db.add_module(module_description);
                }
            }

        }

        if let Some(flatpak_module) = FlatpakModuleDescription::load_from_file(file_path.to_string()) {
            let flatpak_module_path = file_path.replace(&repo_dir, "");
            software_project.flatpak_module_manifests.insert(flatpak_module_path);

            db.add_module(flatpak_module);
        }

    }

    if software_project.supports_flatpak() || !software_project.build_systems.is_empty() {
        db.add_project(software_project);
    }

    if repo_manifest_count == 0 {
        log::info!("Repo at {} had no Flatpak manifest.", repo_url);
    } else {
        log::info!("Repo at {} had {} Flatpak manifests.", repo_url, repo_manifest_count);
    }
    return mined_repos_urls;
}
