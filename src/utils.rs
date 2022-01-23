use std::env;
use std::fs;
use std::io::{stdin, stdout, Write};
use std::path::Path;
use std::process::{Command, Stdio};

use lazy_static::lazy_static;
use regex::Regex;
use uuid::Uuid;

lazy_static! {
    static ref GIT_URL_COLON_REGEX: Regex = Regex::new(r"git://(.+)").unwrap();
    static ref GIT_URL_AT_REGEX: Regex = Regex::new(r"git@(.+)").unwrap();
    // FIXME not sure why GitHub has a different scheme for the git URL. Are there other
    // providers that use this scheme?
    static ref GITHUB_URL_REGEX: Regex = Regex::new(r"git@(.+):(.+)").unwrap();
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
    log::debug!("Getting initial commit for repo at {}", repo_path);

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

///```
///let mut reverse_dns = fpm::utils::repo_url_to_reverse_dns("https://github.com/louib/fpm.git");
///assert_eq!(reverse_dns, "com.github.louib.fpm");
///reverse_dns = fpm::utils::repo_url_to_reverse_dns("https://gitlab.com/louib/fpm.git");
///assert_eq!(reverse_dns, "com.gitlab.louib.fpm");
///reverse_dns = fpm::utils::repo_url_to_reverse_dns("https://git.savannah.gnu.org/cgit/make.git");
///assert_eq!(reverse_dns, "org.gnu.savannah.git.cgit.make");
///reverse_dns = fpm::utils::repo_url_to_reverse_dns("https://gitlab.freedesktop.org/xorg/lib/libxmu");
///assert_eq!(reverse_dns, "org.freedesktop.gitlab.xorg.lib.libxmu");
///```
pub fn repo_url_to_reverse_dns(repo_url: &str) -> String {
    if !repo_url.starts_with("https://") {
        panic!("Only supports https urls: {}", repo_url);
    }
    let mut sanitized_url = repo_url[8..].to_string();

    if repo_url.ends_with(".git") {
        // Removing the .git at the end of the url.
        // There has to be a better way to do this...
        // But rust has no negative index for the list
        // comprehension.
        sanitized_url.pop();
        sanitized_url.pop();
        sanitized_url.pop();
        sanitized_url.pop();
    }

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
    let file_paths = match fpm_core::utils::get_all_paths(std::path::Path::new(dir_path)) {
        Ok(paths) => paths,
        Err(message) => {
            return Err(format!(
                "Could not get file paths for dir {}: {}",
                dir_path, message
            ))
        }
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

        if !flatpak_rs::application::FlatpakApplication::file_path_matches(file_path) {
            continue;
        }
        log::debug!("Found candidate Flatpak manifest {}", file_path);
        response.push(file_path.to_string());
    }
    return Ok(response);
}
