use std::env;
use std::fs;
use std::io::{stdin, stdout, Write};
use std::path::Path;
use std::process::{Command, Stdio};

use uuid::Uuid;

// Gets the path the repos should be located at.
// FIXME not sure this function belongs in utils...
pub fn get_repos_dir_path() -> String {
    if let Ok(path) = env::var("FPM_CLONED_REPOS_DIR") {
        return path.to_string();
    }
    "/tmp".to_string()
}

pub fn clone_git_repo(repo_url: &str) -> Result<String, String> {
    let project_id = repo_url_to_reverse_dns(repo_url);
    let repos_dir = get_repos_dir_path();
    let repo_dir = format!("{}/{}", repos_dir, project_id);
    if Path::new(&repo_dir).is_dir() {
        return Ok(repo_dir);
    }
    if let Err(e) = fs::create_dir(&repo_dir) {
        return Err(e.to_string());
    }

    println!("Cloning repo {}", repo_url);
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

    return Ok(Path::new(archive_path).parent().unwrap().to_str().unwrap().to_string());
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

    println!("Getting file at {}", file_url);
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
    println!("Getting initial commit for repo at {}", repo_path);

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

pub fn get_all_paths(dir: &Path) -> Result<Vec<std::path::PathBuf>, String> {
    let mut all_paths: Vec<std::path::PathBuf> = vec![];

    let dir_entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(err) => return Ok(vec![]),
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

pub fn normalize_name(name: &String) -> String {
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
    if !repo_url.ends_with(".git") {
        panic!("Only supports git repositories: {}", repo_url);
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
    let file_paths = match get_all_paths(std::path::Path::new("./")) {
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
