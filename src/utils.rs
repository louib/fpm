use std::path::Path;

pub fn get_candidate_flatpak_manifests(dir_path: &str) -> Result<Vec<String>, String> {
    let mut response: Vec<String> = vec![];
    let file_paths = match fpm_core::utils::get_all_paths(Path::new(dir_path)) {
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

        if !flatpak_rs::application::FlatpakApplication::file_path_matches(file_path) {
            continue;
        }
        log::debug!("Found candidate Flatpak manifest {}", file_path);
        response.push(file_path.to_string());
    }
    return Ok(response);
}
