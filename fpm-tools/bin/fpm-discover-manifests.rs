use std::collections::BTreeMap;
use std::collections::HashSet;
use std::env;
use std::fs;
use std::path;
use std::process::exit;

use fpm::db::Database;
use fpm::flatpak_manifest::{FlatpakManifest, FlatpakModule, FlatpakModuleDescription};

fn main() {
    fpm::logger::init();

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        panic!("Requires 1 argument: the list of sources to import from, or `all` for all the sources.");
    }

    let sources = &args[1];

    let mut repos_by_source: BTreeMap<String, HashSet<String>> = BTreeMap::new();

    if sources.contains("github-flathub-org") || sources.eq("all") {
        repos_by_source.insert("github-flathub-org".to_string(), HashSet::new());
        let repos_urls = &match get_github_org_repos("flathub") {
            Ok(r) => r,
            Err(e) => panic!(e),
        };
        for repo_url in repos_urls.split("\n") {
            repos_by_source
                .get_mut("github-flathub-org")
                .unwrap()
                .insert(repo_url.to_string());
        }
    }

    if sources.contains("github-elementary-org") || sources.eq("all") {
        repos_by_source.insert("github-elementary-org".to_string(), HashSet::new());
        let repos_urls = &match get_github_org_repos("elementary") {
            Ok(r) => r,
            Err(e) => panic!(e),
        };
        for repo_url in repos_urls.split("\n") {
            repos_by_source
                .get_mut("github-elementary-org")
                .unwrap()
                .insert(repo_url.to_string());
        }
    }

    if sources.contains("github-endless-org") || sources.eq("all") {
        repos_by_source.insert("github-endless-org".to_string(), HashSet::new());
        let repos_urls = &match get_github_org_repos("endlessm") {
            Ok(r) => r,
            Err(e) => panic!(e),
        };
        for repo_url in repos_urls.split("\n") {
            repos_by_source
                .get_mut("github-endless-org")
                .unwrap()
                .insert(repo_url.to_string());
        }
    }

    if sources.contains("gnome-gitlab-instance") || sources.eq("all") {
        repos_by_source.insert("gnome-gitlab-instance".to_string(), HashSet::new());
        let repos_urls = &match get_gitlab_repos("gitlab.gnome.org", "FPM_GNOME_GITLAB_TOKEN") {
            Ok(r) => r,
            Err(e) => panic!(e),
        };
        for repo_url in repos_urls.split("\n") {
            repos_by_source
                .get_mut("gnome-gitlab-instance")
                .unwrap()
                .insert(repo_url.to_string());
        }
    }

    if sources.contains("purism-gitlab-instance") || sources.eq("all") {
        repos_by_source.insert("purism-gitlab-instance".to_string(), HashSet::new());
        let repos_urls = &match get_gitlab_repos("source.puri.sm", "FPM_PURISM_GITLAB_TOKEN") {
            Ok(r) => r,
            Err(e) => panic!(e),
        };
        for repo_url in repos_urls.split("\n") {
            repos_by_source
                .get_mut("purism-gitlab-instance")
                .unwrap()
                .insert(repo_url.to_string());
        }
    }

    if sources.contains("debian-gitlab-instance") || sources.eq("all") {
        repos_by_source.insert("debian-gitlab-instance".to_string(), HashSet::new());
        let repos_urls = &match get_gitlab_repos("salsa.debian.org", "FPM_DEBIAN_GITLAB_TOKEN") {
            Ok(r) => r,
            Err(e) => panic!(e),
        };
        for repo_url in repos_urls.split("\n") {
            repos_by_source
                .get_mut("debian-gitlab-instance")
                .unwrap()
                .insert(repo_url.to_string());
        }
    }

    if sources.contains("xdg-gitlab-instance") || sources.eq("all") {
        repos_by_source.insert("xdg-gitlab-instance".to_string(), HashSet::new());
        let repos_urls = &match get_gitlab_repos("gitlab.freedesktop.org", "FPM_XDG_GITLAB_TOKEN") {
            Ok(r) => r,
            Err(e) => panic!(e),
        };
        for repo_url in repos_urls.split("\n") {
            repos_by_source
                .get_mut("xdg-gitlab-instance")
                .unwrap()
                .insert(repo_url.to_string());
        }
    }

    if sources.contains("kde-gitlab-instance") || sources.eq("all") {
        repos_by_source.insert("kde-gitlab-instance".to_string(), HashSet::new());
        let repos_urls = &match get_gitlab_repos("invent.kde.org", "FPM_KDE_GITLAB_TOKEN") {
            Ok(r) => r,
            Err(e) => panic!(e),
        };
        for repo_url in repos_urls.split("\n") {
            repos_by_source
                .get_mut("kde-gitlab-instance")
                .unwrap()
                .insert(repo_url.to_string());
        }
    }

    if sources.contains("gitlab-search-flatpak") || sources.eq("all") {
        repos_by_source.insert("gitlab-search-flatpak".to_string(), HashSet::new());
        let repos_urls = &match search_gitlab("flatpak") {
            Ok(r) => r,
            Err(e) => panic!(e),
        };
        for repo_url in repos_urls.split("\n") {
            repos_by_source
                .get_mut("gitlab-search-flatpak")
                .unwrap()
                .insert(repo_url.to_string());
        }
    }

    if sources.contains("gitlab-search-flathub") || sources.eq("all") {
        repos_by_source.insert("gitlab-search-flathub".to_string(), HashSet::new());
        let repos_urls = &match search_gitlab("flathub") {
            Ok(r) => r,
            Err(e) => panic!(e),
        };
        for repo_url in repos_urls.split("\n") {
            repos_by_source
                .get_mut("gitlab-search-flathub")
                .unwrap()
                .insert(repo_url.to_string());
        }
    }

    if sources.contains("github-search-flatpak") || sources.eq("all") {
        repos_by_source.insert("github-search-flatpak".to_string(), HashSet::new());
        let repos_urls = &match search_github("flatpak") {
            Ok(r) => r,
            Err(e) => panic!(e),
        };
        for repo_url in repos_urls.split("\n") {
            repos_by_source
                .get_mut("github-search-flatpak")
                .unwrap()
                .insert(repo_url.to_string());
        }
    }

    if sources.contains("github-search-flathub") || sources.eq("all") {
        repos_by_source.insert("github-search-flathub".to_string(), HashSet::new());
        let repos_urls = &match search_github("flathub") {
            Ok(r) => r,
            Err(e) => panic!(e),
        };
        for repo_url in repos_urls.split("\n") {
            repos_by_source
                .get_mut("github-search-flathub")
                .unwrap()
                .insert(repo_url.to_string());
        }
    }

    for (repo_source, repos_urls) in repos_by_source {
        let mut db = Database::get_database();
        mine_repositories(&mut db, &repo_source, repos_urls);
    }

    exit(0);
}

pub fn mine_repositories(db: &mut Database, source: &str, repos_urls: HashSet<String>) {
    let mut next_repos_urls_to_mine: HashSet<String> = HashSet::new();

    for repo_url in repos_urls {
        if repo_url.trim().is_empty() {
            continue;
        }

        // We handle the shared modules separately.
        if repo_url.contains("flathub/shared-modules") {
            continue;
        }

        // Found when searching for `flathub` on GitHub.com
        // Too big to be processed.
        if repo_url.contains("fastrizwaan/winepak") {
            continue;
        }

        // Found when searching for `flathub` on GitHub.com
        // Too big to be processed.
        if repo_url.contains("usrbinkat/ocp-mini-stack") {
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

        // For some reason, the valvesoftware.Steam.CompatibilityTool.Proton
        // project, found in the Flathub org, causes an infinite loop when we
        // try to clone it...
        if repo_url.contains("CompatibilityTool.Proton") {
            continue;
        }

        let project_id = fpm::utils::repo_url_to_reverse_dns(&repo_url);
        if let Some(project) = db.get_project(&project_id) {
            if project.sources.contains(source) {
                log::info!("Repo {} was already mined", &repo_url);
            } else {
                log::info!(
                    "Repo {} was mined from a different source. Adding current source.",
                    &repo_url
                );
                project.sources.insert(source.to_string());
            }
            continue;
        }

        let mined_repos_urls = mine_repository(db, source, &repo_url);

        for mined_repo_url in mined_repos_urls {
            next_repos_urls_to_mine.insert(mined_repo_url);
        }
    }

    if !next_repos_urls_to_mine.is_empty() {
        log::warn!(
            "There are {} other repositories to mine!!!",
            next_repos_urls_to_mine.len()
        );
        mine_repositories(db, "recursive_discovery", next_repos_urls_to_mine);
    }
}

pub fn mine_repository(db: &mut Database, repo_source: &str, repo_url: &str) -> Vec<String> {
    log::info!("Mining repo at {} from {}.", repo_url, repo_source);

    let mut software_project = fpm::projects::SoftwareProject::default();
    software_project.id = fpm::utils::repo_url_to_reverse_dns(repo_url);
    software_project.vcs_urls.insert(repo_url.to_string());
    software_project.sources.insert(repo_source.to_string());

    let mut mined_repos_urls: Vec<String> = vec![];
    let repo_dir = match fpm::utils::clone_git_repo(&repo_url) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Could not clone repo {}: {}", &repo_url, e);
            return mined_repos_urls;
        }
    };

    if let Ok(hashes) = fpm::utils::get_git_repo_root_hashes(&repo_dir) {
        software_project.root_hashes = hashes;
    }

    let repo_file_paths = match fpm::utils::get_all_paths(path::Path::new(&repo_dir)) {
        Ok(paths) => paths,
        Err(message) => {
            log::error!("Could not get all file paths for {}!", repo_dir);
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

        if let Some(build_system) = fpm::build_systems::get_build_system(file_path.to_string()) {
            log::debug!("Detected buildsystem {} for repo {}", build_system, repo_url);
            software_project.build_systems.insert(build_system);
        }

        if let Some(flatpak_manifest) = FlatpakManifest::load_from_file(file_path.to_string()) {
            let flatpak_manifest_path = file_path.replace(&repo_dir, "");
            software_project
                .flatpak_app_manifests
                .insert(flatpak_manifest_path);

            for module in &flatpak_manifest.modules {
                for url in module.get_all_repos_urls() {
                    if url.ends_with(".git") && url.starts_with("https://") {
                        mined_repos_urls.push(url);
                    }
                }
            }
        }

        if let Some(flatpak_module) = FlatpakModuleDescription::load_from_file(file_path.to_string()) {
            let flatpak_module_path = file_path.replace(&repo_dir, "");
            software_project
                .flatpak_module_manifests
                .insert(flatpak_module_path);
        }
    }

    db.add_project(software_project);
    return mined_repos_urls;
}

/// Search for flatpak and flathub related repos on gitlab.com and
/// return their URLs, one on each line.
pub fn search_gitlab(search_term: &str) -> Result<String, String> {
    let gitlab_repos_search_dump_path = format!(
        "{}/gitlab_repo_search_{}.txt",
        Database::get_repos_db_path(),
        search_term
    );
    let gitlab_repos_search_dump_path = path::Path::new(&gitlab_repos_search_dump_path);

    // Reuse the dump if it exists.
    if gitlab_repos_search_dump_path.is_file() {
        log::info!(
            "Dump of the GitLab search for `{}` exists, not fetching.",
            &search_term
        );
        return match fs::read_to_string(gitlab_repos_search_dump_path) {
            Ok(content) => Ok(content),
            Err(e) => Err(e.to_string()),
        };
    }

    log::info!("Searching for {} on GitLab.", &search_term);
    let github_repos = fpm_tools::hubs::gitlab::search_repos(&search_term);
    log::info!(
        "Search for {} returned {} repos.",
        &search_term,
        github_repos.len()
    );

    let mut gitlab_repos_search_dump = "".to_string();
    for github_repo in &github_repos {
        let repo_url = &github_repo.http_url_to_repo;
        gitlab_repos_search_dump += &format!("{}\n", repo_url);
    }

    if !gitlab_repos_search_dump.is_empty() {
        match fs::write(gitlab_repos_search_dump_path, &gitlab_repos_search_dump) {
            Ok(_) => {}
            Err(e) => {
                log::warn!(
                    "Could not save the dump for GitLab search to {}: {}.",
                    gitlab_repos_search_dump_path.display(),
                    e
                );
            }
        };
    }

    Ok(gitlab_repos_search_dump)
}

/// Search for flatpak and flathub related repos on github.com and
/// return their URLs, one on each line.
pub fn search_github(search_term: &str) -> Result<String, String> {
    // TODO clean up the search term.
    let github_repos_search_dump_path = format!(
        "{}/github_repo_search_{}.txt",
        Database::get_repos_db_path(),
        search_term
    );
    let github_repos_search_dump_path = path::Path::new(&github_repos_search_dump_path);

    // Reuse the dump if it exists.
    if github_repos_search_dump_path.is_file() {
        log::info!(
            "Dump of the GitHub search for `{}` exists, not fetching.",
            &search_term
        );
        return match fs::read_to_string(github_repos_search_dump_path) {
            Ok(content) => Ok(content),
            Err(e) => Err(e.to_string()),
        };
    }

    log::info!("Searching for {} on GitHub.", &search_term);
    let github_repos = fpm_tools::hubs::github::search_repos(&search_term);
    log::info!(
        "Search for {} returned {} repos.",
        &search_term,
        github_repos.len()
    );

    let mut github_repos_search_dump = "".to_string();
    for github_repo in &github_repos {
        let repo_url = github_repo.get_git_url();
        github_repos_search_dump += &format!("{}\n", repo_url);
    }

    if !github_repos_search_dump.is_empty() {
        match fs::write(github_repos_search_dump_path, &github_repos_search_dump) {
            Ok(_) => {}
            Err(e) => {
                log::warn!(
                    "Could not save the dump for GitHub search to {}: {}.",
                    github_repos_search_dump_path.display(),
                    e
                );
            }
        };
    }

    Ok(github_repos_search_dump)
}

/// Gets all the repositories' URLs associated with a specific Debian (apt) repository.
pub fn get_debian_repos(debian_repo_name: &str, debian_sources_url: &str) -> Result<String, String> {
    let debian_repos_dump_path = format!("{}/{}.txt", Database::get_repos_db_path(), debian_repo_name);
    let debian_repos_dump_path = path::Path::new(&debian_repos_dump_path);

    // Reuse the dump if it exists.
    if debian_repos_dump_path.is_file() {
        log::info!(
            "Dump of the repos at GitLab instance {} exists, not fetching.",
            &debian_sources_url
        );
        return match fs::read_to_string(debian_repos_dump_path) {
            Ok(content) => Ok(content),
            Err(e) => Err(e.to_string()),
        };
    }

    log::info!(
        "Fetching sources for Debian repo {} at {}.",
        &debian_repo_name,
        &debian_sources_url
    );
    let debian_repos = match fpm_tools::hubs::deb::get_all_repos(&debian_sources_url) {
        Ok(r) => r,
        Err(e) => return Err(e),
    };
    log::info!(
        "There are {} Debian repos at {}.",
        debian_repos.len(),
        &debian_sources_url
    );

    let mut debian_repos_dump = "".to_string();
    for debian_repo_url in &debian_repos {
        debian_repos_dump += &format!("{}\n", debian_repo_url);
    }

    if !debian_repos_dump.is_empty() {
        match fs::write(debian_repos_dump_path, &debian_repos_dump) {
            Ok(_) => {}
            Err(e) => {
                log::warn!(
                    "Could not save the Debian repos dump to {}: {}.",
                    debian_repos_dump_path.display(),
                    e
                );
            }
        };
    }

    Ok(debian_repos_dump)
}

/// Gets all the repositories' URLs for a specific GitLab instance, one on each line.
pub fn get_gitlab_repos(
    gitlab_instance_url: &str,
    gitlab_instance_auth_token_name: &str,
) -> Result<String, String> {
    let gitlab_instance_dump_key = gitlab_instance_url.replace('.', "_");

    let gitlab_instance_repos_dump_path = format!(
        "{}/{}.txt",
        Database::get_repos_db_path(),
        gitlab_instance_dump_key
    );
    let gitlab_instance_repos_dump_path = path::Path::new(&gitlab_instance_repos_dump_path);

    // Reuse the dump if it exists.
    if gitlab_instance_repos_dump_path.is_file() {
        log::info!(
            "Dump of the repos at GitLab instance {} exists, not fetching.",
            &gitlab_instance_url
        );
        return match fs::read_to_string(gitlab_instance_repos_dump_path) {
            Ok(content) => Ok(content),
            Err(e) => Err(e.to_string()),
        };
    }

    log::info!("Fetching repos from GitLab at {}.", &gitlab_instance_url);
    let gitlab_repos =
        fpm_tools::hubs::gitlab::get_all_repos(&gitlab_instance_url, &gitlab_instance_auth_token_name);
    log::info!(
        "There are {} GitLab repos at {}.",
        gitlab_repos.len(),
        &gitlab_instance_url
    );

    let mut gitlab_repos_dump = "".to_string();
    for gitlab_repo in &gitlab_repos {
        let repo_url = &gitlab_repo.http_url_to_repo;
        gitlab_repos_dump += &format!("{}\n", repo_url);
    }

    if !gitlab_repos_dump.is_empty() {
        match fs::write(gitlab_instance_repos_dump_path, &gitlab_repos_dump) {
            Ok(_) => {}
            Err(e) => {
                log::warn!(
                    "Could not save the GitLab repos dump to {}: {}.",
                    gitlab_instance_repos_dump_path.display(),
                    e
                );
            }
        };
    }

    Ok(gitlab_repos_dump)
}

/// Gets all the repositories' URLs for a github.com organization,
/// one on each line.
pub fn get_github_org_repos(org_name: &str) -> Result<String, String> {
    let org_repos_dump_path = format!("{}/{}.txt", Database::get_repos_db_path(), org_name);
    let org_repos_dump_path = path::Path::new(&org_repos_dump_path);

    // Reuse the dump if it exists.
    if org_repos_dump_path.is_file() {
        log::info!("Dump of {} repos exists, not fetching from GitHub.", org_name);
        return match fs::read_to_string(org_repos_dump_path) {
            Ok(content) => Ok(content),
            Err(e) => Err(e.to_string()),
        };
    }

    log::info!("Fetching {} repos from GitHub.", org_name);
    let org_repos = fpm_tools::hubs::github::get_org_repos(org_name);
    log::info!("There are {} {} repos.", org_repos.len(), org_name);

    let mut org_repos_dump = "".to_string();
    for org_repo in &org_repos {
        let repo_url = &org_repo.get_git_url();
        org_repos_dump += &format!("{}\n", repo_url);
    }

    if !org_repos_dump.is_empty() {
        match fs::write(org_repos_dump_path, &org_repos_dump) {
            Ok(_) => {}
            Err(e) => {
                log::warn!(
                    "Could not save the {} repos dump to {}: {}.",
                    org_name,
                    org_repos_dump_path.display(),
                    e
                );
            }
        };
    }

    Ok(org_repos_dump)
}
