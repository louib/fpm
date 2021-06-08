use std::fs::File;
use std::io::{self, prelude::*, BufReader};
use std::path;

pub fn get_all_repos(repo_sources_url: &str) -> Result<Vec<String>, String> {
    log::info!("Getting debian repos {}", repo_sources_url);
    let mut repos_urls: Vec<String> = vec![];

    let debian_sources_file_path = match fpm::utils::fetch_file(repo_sources_url) {
        Ok(p) => p,
        Err(e) => return Err(e),
    };

    let debian_sources_dir_path = match fpm::utils::uncompress(&debian_sources_file_path) {
        Ok(p) => p,
        Err(e) => return Err(e),
    };

    log::info!("Getting all paths in {}", debian_sources_dir_path);
    let debian_sources_file_paths = match fpm::utils::get_all_paths(path::Path::new(&debian_sources_dir_path))
    {
        Ok(paths) => paths,
        Err(message) => return Err(message),
    };

    for file_path in &debian_sources_file_paths {
        if !file_path.is_file() {
            continue;
        }

        let file_path = match file_path.to_str() {
            Some(f) => f,
            None => continue,
        };

        if file_path.ends_with("gz") || file_path.ends_with("xz") {
            continue;
        }

        log::info!("Parsing Debian source file {}", file_path);
        let file = File::open(file_path).unwrap();
        let reader = BufReader::new(file);

        for (index, line) in reader.lines().enumerate() {
            let line = line.unwrap(); // Ignore errors.
            let line = line.trim();
            if line.starts_with("Vcs-Git") {
                let parts: Vec<&str> = line.split(" ").collect();
                let url = parts.get(1).unwrap();
                repos_urls.push(url.to_string());
            }
        }
    }

    Ok(repos_urls)
}
