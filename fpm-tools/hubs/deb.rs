use std::path;

pub fn get_all_repos(repo_sources_url: &str) -> Result<Vec<String>, String> {
    log::info!("Getting debian repos {}", repo_sources_url);

    let debian_sources_file_path = match fpm::utils::fetch_file(repo_sources_url) {
        Ok(p) => p,
        Err(e) => return Err(e),
    };

    let debian_sources_dir_path = match fpm::utils::uncompress(&debian_sources_file_path) {
        Ok(p) => p,
        Err(e) => return Err(e),
    };

    let debian_sources_file_paths = match fpm::utils::get_all_paths(path::Path::new(&debian_sources_dir_path)) {
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
        log::info!("Parsing Debian source file {}", file_path);
    }


    Ok(vec![])
}
