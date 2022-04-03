use flatpak_rs::module::FlatpakModule;

pub const CRATES_IO_URL: &str = "https://static.crates.io/crates";

pub struct CargoLock {
    pub version: String,
}

pub struct CargoLockPackage {
    pub name: String,
    pub source: String,
    pub checksum: String,
    pub version: String,
    pub dependencies: Vec<String>,
}

pub fn get_modules_from_manifest(manifest_path: &str) -> Vec<FlatpakModule> {
    vec![]
}
