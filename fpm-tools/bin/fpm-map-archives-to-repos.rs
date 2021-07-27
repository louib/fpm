use std::collections::BTreeMap;
use std::collections::HashSet;
use std::fs;
use std::path;

use lazy_static::lazy_static;

lazy_static! {
    static ref CANDIDATE_README_NAMES: Vec<String> = vec![
        "README".to_string(),
        "README.md".to_string(),
        "README.txt".to_string(),
    ];
}

fn main() {
    let mut exact_matches: i64 = 0;
    let mut infered_matches: i64 = 0;
    let mut missing_semver: i64 = 0;
    let mut missing_project_name: i64 = 0;

    let mut mapping: BTreeMap<String, String> = BTreeMap::new();

    fpm::logger::init();
    let db = fpm::db::Database::get_database();

    if db.indexed_projects.len() == 0 {
        panic!("There are no projects in the database!");
    }

    let mut git_urls_from_manifests: HashSet<String> = HashSet::new();
    let mut archive_urls_from_manifests: HashSet<String> = HashSet::new();

    let git_urls_dump_path = format!("{}/git_urls_from_manifests.txt", fpm::db::Database::get_db_path());
    let git_urls_dump_path = path::Path::new(&git_urls_dump_path);
    if !git_urls_dump_path.is_file() {
        panic!(
            "Could not find git urls dump at {}.",
            git_urls_dump_path.to_str().unwrap()
        );
    }
    let git_urls_dump = match fs::read_to_string(git_urls_dump_path) {
        Ok(content) => content,
        Err(e) => panic!(e.to_string()),
    };
    for git_url in git_urls_dump.split("\n") {
        git_urls_from_manifests.insert(git_url.to_string());
    }
    log::info!(
        "Loaded {} git urls from the manifests",
        git_urls_from_manifests.len()
    );

    let archive_urls_dump_path = format!(
        "{}/archive_urls_from_manifests.txt",
        fpm::db::Database::get_db_path()
    );
    let archive_urls_dump_path = path::Path::new(&archive_urls_dump_path);
    if !archive_urls_dump_path.is_file() {
        panic!(
            "Could not find git urls dump at {}.",
            archive_urls_dump_path.to_str().unwrap()
        );
    }
    let archive_urls_dump = match fs::read_to_string(archive_urls_dump_path) {
        Ok(content) => content,
        Err(e) => panic!(e.to_string()),
    };
    for archive_url in archive_urls_dump.split("\n") {
        archive_urls_from_manifests.insert(archive_url.to_string());
    }
    log::info!(
        "Loaded {} archive urls from the manifests",
        archive_urls_from_manifests.len()
    );

    for archive_url in &archive_urls_from_manifests {
        if fpm::utils::get_semver_from_archive_url(&archive_url).is_none() {
            missing_semver += 1;
            continue;
        }
        if fpm::utils::get_project_name_from_archive_url(&archive_url).is_none() {
            missing_project_name += 1;
            continue;
        }
        if let Some(git_url) = fpm::utils::get_git_url_from_archive_url(archive_url) {
            exact_matches += 1;
            mapping.insert(archive_url.to_string(), git_url);
        } else if let Some(git_url) = infer_git_url_from_archive(archive_url, &git_urls_from_manifests) {
            infered_matches += 1;
            mapping.insert(archive_url.to_string(), git_url);
        } else {
            println!("Could not find git URL for {}.", archive_url);
        }
    }

    println!(
        "Archive URLs with missing semver: {:.2}% ({}/{})",
        (missing_semver as f64 / archive_urls_from_manifests.len() as f64) * 100.0,
        missing_semver,
        archive_urls_from_manifests.len(),
    );
    println!(
        "Archive URLs with missing project name: {:.2}% ({}/{})",
        (missing_project_name as f64 / archive_urls_from_manifests.len() as f64) * 100.0,
        missing_project_name,
        archive_urls_from_manifests.len(),
    );
    println!(
        "Archive URLs with direct git URL mapping: {:.2}% ({}/{})",
        (exact_matches as f64 / archive_urls_from_manifests.len() as f64) * 100.0,
        exact_matches,
        archive_urls_from_manifests.len(),
    );
    println!(
        "Archive URLs with infered git URL: {:.2}% ({}/{})",
        (infered_matches as f64 / archive_urls_from_manifests.len() as f64) * 100.0,
        infered_matches,
        archive_urls_from_manifests.len(),
    );
    println!(
        "Total archive URLs with a mapping: {:.2}% ({}/{})",
        (mapping.len() as f64 / archive_urls_from_manifests.len() as f64) * 100.0,
        mapping.len(),
        archive_urls_from_manifests.len(),
    );
}

fn infer_git_url_from_archive(archive_url: &str, candidate_git_urls: &HashSet<String>) -> Option<String> {
    let project_name = fpm::utils::get_project_name_from_archive_url(&archive_url).unwrap();
    for git_url in candidate_git_urls {
        if !git_url.to_lowercase().contains(&project_name.to_lowercase()) {
            continue;
        }
        let git_url_matches = match git_url_matches_archive(git_url, archive_url) {
            Ok(r) => r,
            Err(e) => {
                log::warn!("Could not determine if {} matches {}.", git_url, archive_url);
                continue;
            }
        };
        if git_url_matches {
            return Some(git_url.to_string());
        }
    }

    // TODO search in the archive for other potential git repositories.
    None
}

fn git_url_matches_archive(git_url: &str, archive_url: &str) -> Result<bool, String> {
    // FIXME we should actually just handle that by normalizing the git urls...
    if !git_url.starts_with("https://") {
        return Ok(false);
    }

    let archive_version = fpm::utils::get_semver_from_archive_url(archive_url).unwrap();
    let archive_dir = match fpm::utils::get_and_uncompress_archive(archive_url) {
        Ok(d) => d,
        Err(_) => return Ok(false),
    };
    let git_dir = match fpm::utils::clone_git_repo(git_url) {
        Ok(d) => d,
        Err(_) => return Ok(false),
    };
    let git_tags = match fpm::utils::get_git_repo_tags(&git_dir) {
        Ok(t) => t,
        Err(_) => return Ok(false),
    };
    let mut git_refs: Vec<String> = vec![];
    for git_tag in git_tags {
        if git_tag.contains(&archive_version) {
            git_refs.push(git_tag);
        }
    }
    if git_refs.len() == 0 {
        log::info!(
            "Did not find a git tag for version {} of repo {}.",
            archive_version,
            git_url
        );
        return Ok(false);
    }
    if git_refs.len() > 1 {
        log::info!(
            "Found multiple git tags for version {} of repo {}.",
            archive_version,
            git_url
        );
        return Ok(false);
    }
    let git_ref = git_refs.first().unwrap();
    if let Err(_) = fpm::utils::checkout_git_ref(git_url, &git_ref) {
        return Ok(false);
    };
    for candidate_readme_name in CANDIDATE_README_NAMES.iter() {
        let archive_readme_path = format!("{}/{}", archive_dir, candidate_readme_name);
        let archive_readme_path = path::Path::new(&archive_readme_path);
        let git_readme_path = format!("{}/{}", git_dir, candidate_readme_name);
        let git_readme_path = path::Path::new(&git_readme_path);
        if !archive_readme_path.is_file() || !git_readme_path.is_file() {
            log::debug!(
                "{} was not found in the archive or git repo",
                candidate_readme_name
            );
            continue;
        }
        let archive_readme_content = match fs::read_to_string(archive_readme_path) {
            Ok(c) => c,
            Err(e) => {
                log::error!(
                    "Could not load file {}: {}.",
                    archive_readme_path.to_str().unwrap(),
                    e
                );
                continue;
            }
        };
        let git_readme_content = match fs::read_to_string(git_readme_path) {
            Ok(c) => c,
            Err(e) => {
                log::error!(
                    "Could not load file {}: {}.",
                    git_readme_path.to_str().unwrap(),
                    e
                );
                continue;
            }
        };
        if archive_readme_content.eq(&git_readme_content) {
            return Ok(true);
        }
    }

    Ok(false)
}
