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

        if let Some(flatpak_manifest) =
            flatpak_rs::flatpak_manifest::FlatpakManifest::load_from_file(file_path.to_string())
        {
            eprintln!("Importing modules from app manifest at {}.", &file_path);
        }

        let let Some(flatpak_module) = FlatpakModuleDescription::load_from_file(file_path.to_string()) {
            eprintln!("Importing modules from module manifest at {}.", &file_path);
        }

        // TODO also import sources?
        // FlatpakSourceDescription::load_from_file(file_path.to_string())

    }
}
