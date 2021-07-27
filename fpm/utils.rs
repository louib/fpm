use std::env;
use std::fs;
use std::io::{stdin, stdout, Write};
use std::path::Path;
use std::process::{Command, Stdio};

use lazy_static::lazy_static;
use regex::Regex;
use uuid::Uuid;

lazy_static! {
    static ref SEMVER_REGEX: Regex = Regex::new(r"([0-9]+.[0-9]+.[0-9]+)(-[0-9a-zA-Z_]+)?").unwrap();
}

lazy_static! {
    static ref PROJECT_NAME_REGEX: Regex = Regex::new(r"([0-9a-zA-Z_-]+)-[0-9]+.[0-9]+.[0-9]+").unwrap();
}

lazy_static! {
    static ref GITHUB_PROJECT_REGEX: Regex =
        Regex::new(r"https?://github.com/([0-9a-zA-Z_-]+)/([0-9a-zA-Z_-]+)").unwrap();
}

lazy_static! {
    static ref GITLAB_PROJECT_REGEX: Regex =
        Regex::new(r"https?://gitlab.com/([0-9a-zA-Z_-]+)/([0-9a-zA-Z_-]+)").unwrap();
}

lazy_static! {
    static ref GNOME_GITLAB_PROJECT_REGEX: Regex =
        Regex::new(r"https?://gitlab.gnome.org/([0-9a-zA-Z_-]+)/([0-9a-zA-Z_-]+)").unwrap();
}

lazy_static! {
    static ref PAGURE_PROJECT_REGEX: Regex = Regex::new(r"https://pagure.io/([0-9a-zA-Z_-]+)").unwrap();
}

lazy_static! {
    static ref GNU_PROJECT_REGEX: Regex =
        Regex::new(r"https?://ftp.gnu.org/(?:pub/)?gnu/([0-9a-zA-Z_-]+)").unwrap();
}

lazy_static! {
    static ref NONGNU_RELEASE_REGEX: Regex =
        Regex::new(r"https?://download.savannah.nongnu.org/releases/([0-9a-zA-Z_-]+)").unwrap();
}
lazy_static! {
    static ref NONGNU_PROJECT_REGEX: Regex =
        Regex::new(r"https?://savannah.nongnu.org/download/([0-9a-zA-Z_-]+)").unwrap();
}

lazy_static! {
    static ref BITBUCKET_PROJECT_REGEX: Regex =
        Regex::new(r"https?://bitbucket.org/([0-9a-zA-Z_-]+)/([0-9a-zA-Z_-]+)").unwrap();
}

pub fn get_assets_dir() -> String {
    if let Ok(path) = env::var("FPM_ASSETS_DIR") {
        return path.to_string();
    }
    log::warn!("FPM_ASSETS_DIR is not defined, using /tmp/");
    "/tmp".to_string()
}

pub fn checkout_git_ref(repo_url: &str, git_ref: &str) -> Result<(), String> {
    let project_id = repo_url_to_reverse_dns(repo_url);
    let assets_dir = get_assets_dir();
    let repo_dir = format!("{}/repos/{}", assets_dir, project_id);
    if !Path::new(&repo_dir).is_dir() {
        return Err(format!("{} is not a directory!", repo_dir));
    }

    let git_internal_dir = format!("{}/.git", repo_dir);
    if !Path::new(&git_internal_dir).is_dir() {
        return Err(format!("{} is not a git project!", repo_dir));
    }

    log::info!("Checking out {} in repo {}", git_ref, repo_dir);
    let output = Command::new("git")
        .arg(format!("--git-dir={}/.git", repo_dir).to_owned())
        .arg("checkout")
        .arg("-f")
        .arg(git_ref)
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

pub fn clone_git_repo(repo_url: &str) -> Result<String, String> {
    let project_id = repo_url_to_reverse_dns(repo_url);
    let assets_dir = get_assets_dir();
    let repo_dir = format!("{}/repos/{}", assets_dir, project_id);
    if Path::new(&repo_dir).is_dir() {
        return Ok(repo_dir);
    }
    if let Err(e) = fs::create_dir(&repo_dir) {
        return Err(e.to_string());
    }

    log::info!("Cloning repo {}", repo_url);
    let output = Command::new("git")
        .arg("clone")
        .arg(repo_url)
        .arg(&repo_dir)
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let output = match output.wait_with_output() {
        Ok(o) => o,
        Err(e) => return Err(e.to_string()),
    };
    if !output.status.success() {
        return Err("Could not clone repo.".to_string());
    }

    Ok(repo_dir)
}

/// Uncompress an archive into a new temp directory, and returns
/// that directory's path.
pub fn uncompress(archive_path: &str) -> Result<String, String> {
    if !archive_path.ends_with(".gz") {
        return Err("Currently only supports gz archives".to_string());
    }
    log::info!("Uncompressing archive {}.", archive_path);

    // FIXME how can I send the output of unxz somewhere else? Do I have
    // to change the current working directory?
    let output = Command::new("gzip")
        .arg("-d")
        .arg(archive_path.to_string())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let output = match output.wait_with_output() {
        Ok(o) => o,
        Err(e) => return Err(e.to_string()),
    };
    if !output.status.success() {
        return Err("Could not uncompress file.".to_string());
    }

    return Ok(Path::new(archive_path)
        .parent()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string());
}

pub fn fetch_file(file_url: &str) -> Result<String, String> {
    let new_temp_dir_uuid = Uuid::new_v4();
    let new_temp_dir = format!("/tmp/fpm-{}/", new_temp_dir_uuid);
    if let Err(e) = fs::create_dir(&new_temp_dir) {
        return Err(e.to_string());
    }
    log::info!("Created new temp dir {}.", &new_temp_dir);

    let file_name_parts = file_url.split("/");
    let file_name = file_name_parts.last().unwrap();

    log::info!("Getting file at {}", file_url);
    let output = Command::new("wget")
        .arg(file_url.to_string())
        .arg(format!("-P{}", new_temp_dir))
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let local_file_path = new_temp_dir + &file_name.to_owned();

    let output = match output.wait_with_output() {
        Ok(o) => o,
        Err(e) => return Err(e.to_string()),
    };
    if !output.status.success() {
        return Err("Could not fetch file.".to_string());
    }

    Ok(local_file_path)
}

pub fn get_git_repo_root_hashes(repo_path: &str) -> Result<Vec<String>, String> {
    // FIXME there can actually be more than 1 parentless commit
    // in a git repo, in the case of a merger. A parentless commit
    // can also be found in multiple projects in the case of a fork.
    log::info!("Getting initial commit for repo at {}", repo_path);

    let output = Command::new("git")
        .arg(format!("--git-dir={}/.git", repo_path).to_owned())
        .arg("rev-list")
        .arg("--max-parents=0".to_owned())
        .arg("HEAD")
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let output = match output.wait_with_output() {
        Ok(o) => o,
        Err(e) => return Err(e.to_string()),
    };
    if !output.status.success() {
        return Err("Could not get root hashes.".to_string());
    }
    let all_hashes = match std::str::from_utf8(&output.stdout) {
        Ok(v) => v,
        Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
    };

    Ok(all_hashes
        .split('\n')
        .map(|s| s.trim().to_string())
        .filter(|s| s.len() != 0)
        .collect())
}

pub fn get_git_repo_tags(repo_path: &str) -> Result<Vec<String>, String> {
    log::info!("Getting tags for repo at {}", repo_path);

    let output = Command::new("git")
        .arg(format!("--git-dir={}/.git", repo_path).to_owned())
        .arg("tag")
        .arg("-l")
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let output = match output.wait_with_output() {
        Ok(o) => o,
        Err(e) => return Err(e.to_string()),
    };
    if !output.status.success() {
        return Err("Could not get git tags.".to_string());
    }
    let all_tags = match std::str::from_utf8(&output.stdout) {
        Ok(v) => v,
        Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
    };

    Ok(all_tags
        .split('\n')
        .map(|s| s.trim().to_string())
        .filter(|s| s.len() != 0)
        .collect())
}

pub fn get_and_uncompress_archive(archive_url: &str) -> Result<String, String> {
    let archive_path = archive_url.split("/").last().unwrap();
    let dir_name = normalize_name(archive_path);

    let assets_dir = get_assets_dir();
    let archives_dir = format!("{}/archives", assets_dir);
    let uncompressed_archive_dir = format!("{}/uncompressed_archives/{}", assets_dir, dir_name);

    if Path::new(&uncompressed_archive_dir).is_dir() {
        log::info!("Archive was already uncompressed at {}", uncompressed_archive_dir);
        return Ok(uncompressed_archive_dir);
    }
    if let Err(e) = fs::create_dir(&uncompressed_archive_dir) {
        return Err(e.to_string());
    }

    let archive_destination = format!("{}/{}", archives_dir, archive_path);
    if !Path::new(&archive_destination).is_file() {
        log::info!("Getting archive at {}", archive_url);
        let output = Command::new("curl")
            .arg(archive_url)
            .arg(format!("-o{}", archive_destination))
            .arg("-L")
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        let output = match output.wait_with_output() {
            Ok(o) => o,
            Err(e) => return Err(e.to_string()),
        };
        if !output.status.success() {
            return Err(format!("Could not fetch archive from {}.", archive_url));
        }
    } else {
        log::info!("Already downloaded archive at {}", archive_url);
    }

    let archive_type = match crate::flatpak_manifest::FlatpakSourceDescription::detect_archive_type(archive_url)
    {
        Some(t) => t,
        None => return Err(format!("Could not detect archive type for {}", archive_url)),
    };

    if archive_type.starts_with("tar") {
        let mut tar_flags = "";
        if archive_type == "tar-gzip" {
            tar_flags = "-z";
        } else if archive_type == "tar-compress" {
            tar_flags = "-Z";
        } else if archive_type == "tar-bzip2" {
            tar_flags = "--bzip2";
        } else if archive_type == "tar-lzip" {
            tar_flags = "--lzip";
        } else if archive_type == "tar-lzma" {
            tar_flags = "--lzma";
        } else if archive_type == "tar-lzop" {
            tar_flags = "--lzop";
        } else if archive_type == "tar-xz" {
            tar_flags = "--xz";
        }

        log::info!(
            "Decompressing with `tar` the following archive: {}",
            archive_destination
        );
        log::debug!("Tar flags: {}", tar_flags);
        // TODO should we handle the strip-components option?
        let output = Command::new("tar")
            .arg(format!("--directory={}", uncompressed_archive_dir))
            .arg("--no-same-owner")
            .arg("--strip-components=1")
            .arg("-x")
            .arg(tar_flags)
            .arg(format!("-f{}", archive_destination))
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        let output = match output.wait_with_output() {
            Ok(o) => o,
            Err(e) => return Err(e.to_string()),
        };
        if !output.status.success() {
            return Err(format!("Could not extract archive from {}.", archive_destination));
        }
    }

    Ok("".to_string())
}

pub fn get_all_paths(dir: &Path) -> Result<Vec<std::path::PathBuf>, String> {
    let mut all_paths: Vec<std::path::PathBuf> = vec![];

    let dir_entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(err) => return Err(err.to_string()),
    };
    for entry in dir_entries {
        let entry_path = entry.unwrap().path();
        if entry_path.is_dir() {
            let mut dir_paths: Vec<std::path::PathBuf> = get_all_paths(&entry_path)?;
            all_paths.append(&mut dir_paths);
        } else {
            all_paths.push(entry_path);
        }
    }

    Ok(all_paths)
}

pub fn ask_yes_no_question(question: String) -> bool {
    let mut answer = String::new();
    print!("{}? [Y/n]: ", question);
    let _ = stdout().flush();
    stdin()
        .read_line(&mut answer)
        .expect("Error while reading answer for question.");
    if let Some('\n') = answer.chars().next_back() {
        answer.pop();
    }
    if let Some('\r') = answer.chars().next_back() {
        answer.pop();
    }
    if answer == "Y" || answer == "y" {
        return true;
    }
    return false;
}

pub fn normalize_name(name: &str) -> String {
    let mut response: String = "".to_string();
    for c in name.chars() {
        if c.is_alphabetic() || c.is_numeric() {
            response.push_str(&c.to_string());
            continue;
        }
        // We don't want to add multiple hyphens or dots in a row, and we want
        // to start the name with an alphanum character.
        if response.ends_with("-") || response.ends_with(".") || response.is_empty() {
            continue;
        }
        response.push_str(&c.to_string());
    }
    response
}

// TODO migrate to fpm-tools
pub struct PagedResponse<T> {
    pub next_page_url: Option<String>,
    pub results: Vec<T>,
    pub token: Option<String>,
}

// TODO migrate to fpm-tools
pub struct PagedRequest {
    pub next_page_url: Option<String>,
    pub domain: String,
    pub token: Option<String>,
}

// TODO migrate to fpm-tools
/// See https://www.w3.org/wiki/LinkHeader
///```
///let link_header = r###"
///<https://gitlab.gnome.org/api/v4/projects?page=4&per_page=100>; rel="prev",
///<https://gitlab.gnome.org/api/v4/projects?page=6&per_page=100>; rel="next",
///<https://gitlab.gnome.org/api/v4/projects?page=1&per_page=100>; rel="first",
///<https://gitlab.gnome.org/api/v4/projects?page=118&per_page=100>; rel="last"
///"###;
///assert_eq!(
///  fpm::utils::get_next_page_url(link_header),
///  Some("https://gitlab.gnome.org/api/v4/projects?page=6&per_page=100".to_string()),
///);
///assert_eq!(
///  fpm::utils::get_next_page_url(""),
///  None,
///);
///
///```
pub fn get_next_page_url(link_header: &str) -> Option<String> {
    log::debug!("Getting next page from header {}.", link_header);
    for link in link_header.split(",") {
        let mut link_parts = link.split(";");
        let url = match link_parts.next() {
            Some(u) => u,
            None => continue,
        };
        let rel = match link_parts.next() {
            Some(u) => u,
            None => continue,
        };
        if !rel.contains("rel=\"next\"") {
            continue;
        }
        let mut next_page_url = url.trim();
        next_page_url = &next_page_url[1..next_page_url.len() - 1];
        return Some(next_page_url.to_string());
    }
    None
}

///```
///let mut reverse_dns = fpm::utils::repo_url_to_reverse_dns("https://github.com/louib/fpm.git");
///assert_eq!(reverse_dns, "com.github.louib.fpm");
///reverse_dns = fpm::utils::repo_url_to_reverse_dns("https://gitlab.com/louib/fpm.git");
///assert_eq!(reverse_dns, "com.gitlab.louib.fpm");
///reverse_dns = fpm::utils::repo_url_to_reverse_dns("https://git.savannah.gnu.org/cgit/make.git");
///assert_eq!(reverse_dns, "org.gnu.savannah.git.cgit.make");
///```
pub fn repo_url_to_reverse_dns(repo_url: &str) -> String {
    if !repo_url.starts_with("https://") {
        panic!("Only supports https urls: {}", repo_url);
    }
    let mut sanitized_url = repo_url[8..].to_string();
    // Removing the .git at the end of the url.
    // There has to be a better way to do this...
    // But rust has no negative index for the list
    // comprehension.
    sanitized_url.pop();
    sanitized_url.pop();
    sanitized_url.pop();
    sanitized_url.pop();

    let mut repo_url_parts = sanitized_url.split("/");
    let domain = repo_url_parts.next().unwrap();
    let mut reversed_domain: String = "".to_string();

    let domain_parts = domain.split(".");
    for domain_part in domain_parts {
        if reversed_domain.len() == 0 {
            reversed_domain = domain_part.to_string();
        } else {
            reversed_domain = format!("{}.{}", domain_part, reversed_domain);
        }
    }

    let mut next_url_part = repo_url_parts.next();
    while next_url_part.is_some() {
        reversed_domain += ".";
        reversed_domain += next_url_part.unwrap();
        next_url_part = repo_url_parts.next();
    }
    reversed_domain
}

pub fn remove_comments_from_json(json_content: &str) -> String {
    let mut json_content_without_comments = "".to_string();
    let mut is_in_a_comment = false;
    for manifest_line in json_content.split('\n') {
        if manifest_line.trim().starts_with("/*") && manifest_line.trim().ends_with("*/") {
            continue;
        }
        if manifest_line.trim().starts_with("/*") && !is_in_a_comment {
            is_in_a_comment = true;
            continue;
        }
        if manifest_line.trim().ends_with("*/") && is_in_a_comment {
            is_in_a_comment = false;
            continue;
        }
        if is_in_a_comment {
            continue;
        }
        // TODO should we also filter out comments at the end of the lines?
        json_content_without_comments += manifest_line;
    }
    return json_content_without_comments;
}

pub fn get_candidate_flatpak_manifests(dir_path: &str) -> Result<Vec<String>, String> {
    let mut response: Vec<String> = vec![];
    let file_paths = match get_all_paths(std::path::Path::new(dir_path)) {
        Ok(paths) => paths,
        Err(message) => return Err(message),
    };
    for file_path in file_paths.iter() {
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

        if file_path.contains(".flatpak-builder/") {
            continue;
        }

        if !crate::flatpak_manifest::FlatpakManifest::file_path_matches(file_path) {
            continue;
        }
        response.push(file_path.to_string());
    }
    return Ok(response);
}

///```
///let version = fpm::utils::get_semver_from_archive_url(
///  "https://download-fallback.gnome.org/sources/libgda/5.2/libgda-5.2.9.tar.xz"
///);
///assert!(version.is_some());
///assert_eq!(version.unwrap(), "5.2.9");
///
///let version = fpm::utils::get_semver_from_archive_url(
///  "https://download.gnome.org/core/3.28/3.28.2/sources/libgsf-1.14.43.tar.xz"
///);
///assert!(version.is_some());
///assert_eq!(version.unwrap(), "1.14.43");
///
///let version = fpm::utils::get_semver_from_archive_url(
///  "https://download.gnome.org/core/3.28/3.28.2/sources/libgsf-1.14.43.tar.xz"
///);
///assert!(version.is_some());
///assert_eq!(version.unwrap(), "1.14.43");
///
///let version = fpm::utils::get_semver_from_archive_url(
///  "https://github.com/haskell/ghc/releases/download/ghc-8.6.3-release/ghc-8.6.3-armv7-deb8-linux.tar.xz"
///);
///assert!(version.is_some());
///assert_eq!(version.unwrap(), "8.6.3");
///
///let version = fpm::utils::get_semver_from_archive_url(
///  "https://github.com/GNOME/libxml2/archive/v2.9.10.tar.gz"
///);
///assert!(version.is_some());
///assert_eq!(version.unwrap(), "2.9.10");
///
///let version = fpm::utils::get_semver_from_archive_url(
///  "https://github.com/sass/libsass/archive/3.6.4.tar.gz"
///);
///assert!(version.is_some());
///assert_eq!(version.unwrap(), "3.6.4");
///```
pub fn get_semver_from_archive_url(archive_url: &str) -> Option<String> {
    let archive_filename = archive_url.split("/").last().unwrap();
    let captured_groups = match SEMVER_REGEX.captures(archive_filename) {
        Some(g) => g,
        None => return None,
    };
    if captured_groups.len() == 0 {
        return None;
    }
    return Some(captured_groups[1].to_string());
}

///```
///let version = fpm::utils::get_project_name_from_archive_url(
///  "https://download-fallback.gnome.org/sources/libgda/5.2/libgda-5.2.9.tar.xz"
///);
///assert!(version.is_some());
///assert_eq!(version.unwrap(), "libgda");
///
///let version = fpm::utils::get_project_name_from_archive_url(
///  "https://download.gnome.org/core/3.28/3.28.2/sources/libgsf-1.14.43.tar.xz"
///);
///assert!(version.is_some());
///assert_eq!(version.unwrap(), "libgsf");
///```
pub fn get_project_name_from_archive_url(archive_url: &str) -> Option<String> {
    let archive_filename = archive_url.split("/").last().unwrap();
    let captured_groups = match PROJECT_NAME_REGEX.captures(archive_filename) {
        Some(g) => g,
        None => return None,
    };
    if captured_groups.len() == 0 {
        return None;
    }
    return Some(captured_groups[1].to_string());
}

///```
///let git_url = fpm::utils::get_git_url_from_archive_url(
///  "https://github.com/sass/libsass/archive/3.6.4.tar.gz"
///);
///assert!(git_url.is_some());
///assert_eq!(git_url.unwrap(), "https://github.com/sass/libsass.git");
///
///let git_url = fpm::utils::get_git_url_from_archive_url(
///  "https://gitlab.com/rszibele/e-juice-calc/-/archive/1.0.7/e-juice-calc-1.0.7.tar.bz2"
///);
///assert!(git_url.is_some());
///assert_eq!(git_url.unwrap(), "https://gitlab.com/rszibele/e-juice-calc.git");
///
///let git_url = fpm::utils::get_git_url_from_archive_url(
///  "https://gitlab.gnome.org/GNOME/libsecret/-/archive/0.19.1/libsecret-0.19.1.tar.gz"
///);
///assert!(git_url.is_some());
///assert_eq!(git_url.unwrap(), "https://gitlab.gnome.org/GNOME/libsecret.git");
///
///let git_url = fpm::utils::get_git_url_from_archive_url(
///  "https://pagure.io/libaio/archive/libaio-0.3.111/libaio-libaio-0.3.111.tar.gz"
///);
///assert!(git_url.is_some());
///assert_eq!(git_url.unwrap(), "https://pagure.io/libaio.git");
///
///let git_url = fpm::utils::get_git_url_from_archive_url(
///  "https://ftp.gnu.org/pub/gnu/libiconv/libiconv-1.16.tar.gz"
///);
///assert!(git_url.is_some());
///assert_eq!(git_url.unwrap(), "https://git.savannah.gnu.org/git/libiconv.git");
///
///let git_url = fpm::utils::get_git_url_from_archive_url(
///  "http://ftp.gnu.org/gnu/autoconf/autoconf-2.13.tar.gz"
///);
///assert!(git_url.is_some());
///assert_eq!(git_url.unwrap(), "https://git.savannah.gnu.org/git/autoconf.git");
///
///let git_url = fpm::utils::get_git_url_from_archive_url(
///  "https://download.savannah.nongnu.org/releases/openexr/openexr-2.2.1.tar.gz"
///);
///assert!(git_url.is_some());
///assert_eq!(git_url.unwrap(), "https://git.savannah.nongnu.org/git/openexr.git");
///
///let git_url = fpm::utils::get_git_url_from_archive_url(
///  "http://savannah.nongnu.org/download/icoutils/icoutils-0.31.1.tar.bz2"
///);
///assert!(git_url.is_some());
///assert_eq!(git_url.unwrap(), "https://git.savannah.nongnu.org/git/icoutils.git");
///
///let git_url = fpm::utils::get_git_url_from_archive_url(
///  "https://bitbucket.org/Doomseeker/doomseeker/get/1.3.1.tar.bz2"
///);
///assert!(git_url.is_some());
///assert_eq!(git_url.unwrap(), "https://bitbucket.org/Doomseeker/doomseeker.git");
///```
pub fn get_git_url_from_archive_url(archive_url: &str) -> Option<String> {
    if let Some(git_url) = get_github_url_from_archive_url(archive_url) {
        return Some(git_url);
    }
    if let Some(git_url) = get_gitlab_url_from_archive_url(archive_url) {
        return Some(git_url);
    }
    if let Some(git_url) = get_gnome_gitlab_url_from_archive_url(archive_url) {
        return Some(git_url);
    }
    if let Some(git_url) = get_pagure_url_from_archive_url(archive_url) {
        return Some(git_url);
    }
    if let Some(git_url) = get_gnu_url_from_archive_url(archive_url) {
        return Some(git_url);
    }
    if let Some(git_url) = get_nongnu_release_url_from_archive_url(archive_url) {
        return Some(git_url);
    }
    if let Some(git_url) = get_nongnu_project_url_from_archive_url(archive_url) {
        return Some(git_url);
    }
    if let Some(git_url) = get_bitbucket_url_from_archive_url(archive_url) {
        return Some(git_url);
    }
    // The SourceForge git access is documented here
    // https://sourceforge.net/p/forge/documentation/Git/#anonymous-access-read-only
    None
}

pub fn get_github_url_from_archive_url(archive_url: &str) -> Option<String> {
    let captured_groups = match GITHUB_PROJECT_REGEX.captures(archive_url) {
        Some(g) => g,
        None => return None,
    };
    if captured_groups.len() == 0 {
        return None;
    }
    let user_name: String = captured_groups[1].to_string();
    let project_name: String = captured_groups[2].to_string();
    return Some(format!("https://github.com/{}/{}.git", user_name, project_name));
}

pub fn get_gitlab_url_from_archive_url(archive_url: &str) -> Option<String> {
    let captured_groups = match GITLAB_PROJECT_REGEX.captures(archive_url) {
        Some(g) => g,
        None => return None,
    };
    if captured_groups.len() == 0 {
        return None;
    }
    let user_name: String = captured_groups[1].to_string();
    let project_name: String = captured_groups[2].to_string();
    return Some(format!("https://gitlab.com/{}/{}.git", user_name, project_name));
}

pub fn get_gnome_gitlab_url_from_archive_url(archive_url: &str) -> Option<String> {
    let captured_groups = match GNOME_GITLAB_PROJECT_REGEX.captures(archive_url) {
        Some(g) => g,
        None => return None,
    };
    if captured_groups.len() == 0 {
        return None;
    }
    let user_name: String = captured_groups[1].to_string();
    let project_name: String = captured_groups[2].to_string();
    return Some(format!(
        "https://gitlab.gnome.org/{}/{}.git",
        user_name, project_name
    ));
}

pub fn get_pagure_url_from_archive_url(archive_url: &str) -> Option<String> {
    let captured_groups = match PAGURE_PROJECT_REGEX.captures(archive_url) {
        Some(g) => g,
        None => return None,
    };
    if captured_groups.len() == 0 {
        return None;
    }
    let project_name: String = captured_groups[1].to_string();
    return Some(format!("https://pagure.io/{}.git", project_name));
}

pub fn get_gnu_url_from_archive_url(archive_url: &str) -> Option<String> {
    let captured_groups = match GNU_PROJECT_REGEX.captures(archive_url) {
        Some(g) => g,
        None => return None,
    };
    if captured_groups.len() == 0 {
        return None;
    }
    let project_name: String = captured_groups[1].to_string();
    return Some(format!("https://git.savannah.gnu.org/git/{}.git", project_name));
}

pub fn get_nongnu_release_url_from_archive_url(archive_url: &str) -> Option<String> {
    let captured_groups = match NONGNU_RELEASE_REGEX.captures(archive_url) {
        Some(g) => g,
        None => return None,
    };
    if captured_groups.len() == 0 {
        return None;
    }
    let project_name: String = captured_groups[1].to_string();
    return Some(format!(
        "https://git.savannah.nongnu.org/git/{}.git",
        project_name
    ));
}

pub fn get_nongnu_project_url_from_archive_url(archive_url: &str) -> Option<String> {
    let captured_groups = match NONGNU_PROJECT_REGEX.captures(archive_url) {
        Some(g) => g,
        None => return None,
    };
    if captured_groups.len() == 0 {
        return None;
    }
    let project_name: String = captured_groups[1].to_string();
    return Some(format!(
        "https://git.savannah.nongnu.org/git/{}.git",
        project_name
    ));
}

pub fn get_bitbucket_url_from_archive_url(archive_url: &str) -> Option<String> {
    // Bitbucket does not allow anonymous git access by default, so this
    // might fail.
    let captured_groups = match BITBUCKET_PROJECT_REGEX.captures(archive_url) {
        Some(g) => g,
        None => return None,
    };
    if captured_groups.len() == 0 {
        return None;
    }
    let username: String = captured_groups[1].to_string();
    let project_name: String = captured_groups[2].to_string();
    return Some(format!("https://bitbucket.org/{}/{}.git", username, project_name));
}
