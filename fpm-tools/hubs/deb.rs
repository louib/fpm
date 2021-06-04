pub fn get_all_repos(repo_sources_url: &str) -> Result<Vec<String>, String> {
    log::info!("Getting debian repos {}", repo_sources_url);

    let debian_sources_file_path = match fpm::utils::fetch_file(repo_sources_url) {
        Ok(p) => p,
        Err(e) => return Err(e),
    };



    Ok(vec![])
}
