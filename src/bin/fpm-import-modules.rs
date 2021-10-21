use std::env;
use std::path;

fn main() {
    fpm::logger::init();

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        panic!("Requires 1 argument: the path of the directory to use for the import.");
    }

    let path = &args[1];

    let file_paths = match fpm::utils::get_all_paths(path::Path::new(path)) {
        Ok(paths) => paths,
        Err(message) => {
            eprintln!("Could not get the file paths :sad: {}", message);
            return;
        }
    };
    let mut git_urls: Vec<String> = vec![];
    let mut db = fpm::db::Database::get_database();

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

        if let Ok(flatpak_manifest) =
            flatpak_rs::flatpak_manifest::FlatpakManifest::load_from_file(file_path.to_string())
        {
            eprintln!("Importing modules from app manifest at {}.", &file_path);
            for module in flatpak_manifest.get_all_modules_recursively() {
                let mut m = match module {
                    flatpak_rs::flatpak_manifest::FlatpakModule::Description(m) => m,
                    flatpak_rs::flatpak_manifest::FlatpakModule::Path(_) => continue,
                };
                db.add_module(m.clone());
                for git_url in m.get_all_git_urls() {
                    git_urls.push(git_url);
                }
            }
        }

        if let Ok(flatpak_module) =
            flatpak_rs::flatpak_manifest::FlatpakModuleDescription::load_from_file(file_path.to_string())
        {
            eprintln!("Importing modules from module manifest at {}.", &file_path);
            for module in flatpak_module.get_all_modules_recursively() {
                let mut m = match module {
                    flatpak_rs::flatpak_manifest::FlatpakModule::Description(m) => m,
                    flatpak_rs::flatpak_manifest::FlatpakModule::Path(_) => continue,
                };
                db.add_module(m.clone());
                for git_url in m.get_all_git_urls() {
                    git_urls.push(git_url);
                }
            }
        }

        // TODO also import sources?
        // FlatpakSourceDescription::load_from_file(file_path.to_string())
    }

    // TODO here we should normalize using either https or git urls.
    for git_url in &git_urls {
        if !(git_url.starts_with("https://") || git_url.starts_with("git://")) {
            continue;
        }
        let mut git_url = git_url.to_string();
        if git_url.starts_with("git://") {
            git_url = match fpm::utils::git_url_to_https_url(&git_url) {
                Some(u) => u,
                None => {
                    log::error!("Could not convert git url to https url {}", git_url);
                    continue;
                },
            }
        }
        let mut project = fpm::projects::SoftwareProject::default();
        project.vcs_urls.insert(git_url.to_string());
        project.id = fpm::utils::repo_url_to_reverse_dns(&git_url);
        db.add_project(project);
    }
}
