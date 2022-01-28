use std::io::{stdin, stdout, Write};
use std::path::Path;

pub const DEFAULT_FLATPAK_BUILDER_CACHE_DIR: &str = ".flatpak-builder/";
pub const DEFAULT_FLATPAK_BUILDER_OUTPUT_DIR: &str = ".flatpak-builder-out/";
pub const DEFAULT_GIT_CACHE_DIR: &str = ".git/";

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

        if file_path.contains(DEFAULT_GIT_CACHE_DIR) {
            continue;
        }
        if file_path.contains(DEFAULT_FLATPAK_BUILDER_CACHE_DIR) {
            continue;
        }
        if file_path.contains(DEFAULT_FLATPAK_BUILDER_OUTPUT_DIR) {
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
