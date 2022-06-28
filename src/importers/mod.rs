use std::fs;

use flatpak_rs::module::FlatpakModule;

pub mod cargo;
pub mod goget;
pub mod vcpkg;

pub enum PackageManager {
    Cargo,
    GoGet,
    Vcpkg,
}
impl PackageManager {
    pub fn detect_from_manifest_path(manifest_path: &str) -> Option<PackageManager> {
        if manifest_path.ends_with("Cargo.toml") {
            return Some(PackageManager::Cargo);
        }
        None
    }

    pub fn import_packages(&self, manifest_path: &str) -> FlatpakModule {
        match &self {
            PackageManager::Cargo => {
                // This could be made more robust by replacing only the string at the
                // end, but this is good enough nonetheless.
                let cargo_lock_path = manifest_path.replace("Cargo.toml", "Cargo.lock");
                let cargo_lock_content = match fs::read_to_string(&cargo_lock_path) {
                    Ok(c) => c,
                    Err(e) => panic!("Could not read Cargo.lock file at {}: {}", &cargo_lock_path, e),
                };
                return cargo::get_cargo_module(&cargo_lock_content);
            }
            PackageManager::Vcpkg => panic!("Not implemented yet."),
            PackageManager::GoGet => panic!("Not implemented yet."),
        }
    }
}
