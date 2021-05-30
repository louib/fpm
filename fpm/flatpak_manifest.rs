use std::collections::hash_map::DefaultHasher;
use std::collections::BTreeMap;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path;
use std::process::{Command, Stdio};
use std::str;

use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};

const DEFAULT_FLATPAK_BUILDER_CACHE_DIR: &str = ".flatpak-builder";
const DEFAULT_FLATPAK_OUTPUT_DIR: &str = "build";

// Other choices are org.gnome.Platform and org.kde.Platform
const DEFAULT_RUNTIME: &str = "org.freedesktop.Platform";
const DEFAULT_RUNTIME_VERSION: &str = "master";
// Other choices are org.gnome.Sdk and org.kde.Sdk
const DEFAULT_SDK: &str = "org.freedesktop.Sdk";

const FLATHUB_REPO_SPEC: &str = r###"
[Flatpak Repo]
Title=Flathub
Url=https://dl.flathub.org/repo/
Homepage=https://flathub.org/
Comment=Central repository of Flatpak applications
Description=Central repository of Flatpak applications
Icon=https://dl.flathub.org/repo/logo.svg
GPGKey=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
"###;

const GNOME_REPO_SPEC: &str = r###"
[Flatpak Repo]
Title=GNOME Nightly Builds Repository
Url=https://nightly.gnome.org/repo/
Homepage=https://gnome.org/
Description=The latest GNOME flatpak runtimes/apps directly from the gitlab CI. This is highly unstable stuff aimed at GNOME developers/designers to test
GPGKey=mQENBFYSUcEBCAC50sRVDy40A0mF/L877gPpjP2GVunQ+VGd5NY0MPwlSxG2TxM0VwfAjHZvDcWxKV0842bVfAXWzmbxMiVRZKFJMAsjWopsuvFCg14h4ysBJYL0T4gmaTOn49d8WUpyJzN1MeS8GEOVvNUa+w/q+ScW8/cICerzkMSiiQzg86Ph0YTvpsFy/feSPZk5VfY28Nw5204DO6z06+i4HuHm7wu8uSPu3TNrQwSPgqF+CeY1nHnw/LZIY6dRykkWOsOnfyPSd0EK8QYJD6q6i1JOctXutG/gR4GvbprgDagtJQpQmHHaGnoe0qJHOD0TOd+7mEiel0AfyPMwkzcobi5LwfwjABEBAAG0Kk5pZ2h0bHkgYXBwIGF1dG9idWlsZGVyIDxhbGV4bEByZWRoYXQuY29tPokBNwQTAQgAIQUCVhJRwQIbAwULCQgHAgYVCAkKCwIEFgIDAQIeAQIXgAAKCRBqfF1EghcOPVqiCACeD67ypYuBj+fx1Tfm3b9zW7E3g2FI1gLcK1KYva89pE1IUmYiFb6Lk/pWgNZpsbchsXxd7cOh31p66z1R/Mz10XYO5/33z4lrsJYna20ist7Gf3s+f8Wgg31T28DdhRP7ID8mjsZJo2iTutaWEwAAD23ggsEeW1kzMsIiOqzbJ+WXGhQv2kdfXOLgDSJElaH3gXsd60sbgxJVDD7IYxoInbv6C/WXc9+rLcY7zrauAIRoDrMIb5Z3EE7SJ5INhB3A9oM0fVIfGo9M1tNkk/FIbqYne07Rmb+4voA2wyT1wBHsqz5G+FxkFO1+gAyLc3AUHSHVFX7iQd5x4s91w+J5uQENBFYSUcEBCAC2EN17qhMREDRVY7/j6qj3CmQB/9OTNtWexJimpQ/q60a3LrE0KAu10eOV+IeURQcx2CRxHSIX3kAApOSu0xufN/flqWLfE39iLwpcwnp4fsEYphlrw5agkFUzkex2Jx++3Llp4v3FXoM1Fmgo8GzAzKlk0JFqszQf2FA5wupFh0hc3u0cwln3W4IluXGveAoaF1WZTsjPwJ8Qtstpd68801Zb9jHez+8GkBk5aFkxvOuZFvXGRwOyo5O8RyjFFEaRqi6WEBNGYcmsihgSg3fNi1htPY18AqUcaZrBaSQ36RMC7tPbQZQGrg40hWlPrMWvzHDPOusmUYH8xsAHjdufABEBAAGJAR8EGAEIAAkFAlYSUcECGwwACgkQanxdRIIXDj2sjgf+KcheM4QJCVhnxcsv3geUyGRaguswIDqWc6xv6ScqTLgOBSlaVcqrbtVErJXINxa5fb3eX7UmuCVySPkPoip1IBH07+zvKtOlqOYIdFd9oaAqmRxX4iJiNr/70c7Gi09xg8EWY7BKLosLojIns/HawZ7kLO3rRPGjEV0oIavH8jiaAHm6rp6d5awGaZP6c7+ZuB+oFIZT71RsFoNPHhRKId4cC4zdcK1VCzJep6VekpepCmmh264TJdrfa0TEOAxt4DTQKz0gCYIKg8QRpYw/URi/kP15Xk/FbnG8hc2svd+u7mCd3FEvhuf25t7LBmUxsAWCR9HRgdnkFafANaa2JA==
"###;

const KDE_REPO_SPEC: &str = r###"
[Flatpak Repo]
Title=KDE Runtimes
Url=http://distribute.kde.org/flatpak-testing/
Icon=https://distribute.kde.org/flatpak-testing/kdeapp.svg
Comment=The latest version of the KDE runtime. This runtime is unstable and is aimed at KDE developers mainly.
Description=The latest version of the KDE runtime. This runtime is unstable and is aimed at KDE developers mainly.
GPGKey=mQENBFdaDxkBCADFg5SnlJNKTa+qY4Zqhb89O0PQ/NPx/GCK/LRYu7+OAWqQljuKZ/GZUEWG1Fzz5GbOy+EH+hDYq6ReUAM958jsrz8pemmriAB/iQtwRXVXNQcJmeB8b4e6L96lcKTwZDYNPLO4CuGqPPkEF4CYzs+wjkDR6q6KiqBNelVTjTJUd3VBvQlbBhSGZeCXXnJOIYMf2XXETFv8tuoW3SO9dkC018O+xZgF2WIQjHCZK5uChkyF69i3eAtVPRDRoXetH6eI+2HTznhnv3liCu6b2AXCpu5IlnSeoDy2wgmtmYgwu0KzkivKVvaY7qQYSr0JWGioz9NSEsbrhtCb3G3+XuC1ABEBAAG0RUtERSBGbGF0cGFrIChUaGUgZS1tYWlsIGlzIGEgcHVibGljIG1haWxpbmcgbGlzdCkgPGtkZS1kZXZlbEBrZGUub3JnPokBNwQTAQgAIQUCV1oPGQIbAwULCQgHAgYVCAkKCwIEFgIDAQIeAQIXgAAKCRCOITqGYcRb7d2sCACu29H4jzC8bwDB3MMwYTy8nVfeJtCq1LPLnolFG0WMqDtLeOg2q3PdrjjJSquIuPbHTlq+1HWFrEJ3gJ+X26O8bw0acVWdMXPEuJiuQTd6RdWG1y6QpEqAlBVBQ1vF5vCdXrBed6nWodhQ1vQ0iPMzGh1dEyHI9wOyyF3+PCKhq6NY41cftoQSFeXtYgMUL82gq436gVvvqFockavDV407rZkmJflry+f9nNJrBTpNZijd0hi+eVQ0mty4dGoWPAI+1DcR8349vGHCHsCIRRGj0Dra89XKZXJZJDze7LutP7lcY7W2x0alZJogc++wxACos8NldyOvSLuXdrFaMKW4uQENBFdaDxkBCADCXBI4M1iOAcwuNUBeSl85s0VRzIalN2mlyIFw4401Z+heuUXYRrdSUokk+5ea4WiECxe8qw44kLEUVRTBar5xH2pUmmxjupBadzDhr8/Wa2WLj3O4DGPYRBK1A8zNhtL1safxczZ1EukCnIZzstp9gUBqVvAu5ebe3VcAoMYGuqltgxhOS41zDQ7hGyxx+NNvvhCBxjUOV9hmmCo2u0r4Vq28LXiRctEiCKYmgyDj1Hcq86Vlwp7sJ4V4m1Eyewq8IepMzz3zhMpnFnkd03NE5twP/puIwAArzmcLlUed0WOp0YffPqGQe5+NRIJyWkhaxj7BtK8WAVMPmPkW84u/ABEBAAGJAR8EGAEIAAkFAldaDxkCGwwACgkQjiE6hmHEW+3JxQf/bn24l++Nmjj+Vnzi9xZNPKU9DmAQTxigTTBSRkkTLBqjaJn1C0Wuiago7TDrIqGlvA0H1xSYSxiiAnauvxhTEO0o9WNAhbdLotMk3KrysuW/vE+ZRecDoi/2aYX0ANnRG4jDl/2yXYo0+iH9qADkTHYJyhT3U3MDJDjgAQC03jnYe+9Hc6N801eGw/sQvSjLEGsne+nEWwMmhpj8puVCLEoiSwd76fnhcaJuvOwgrldr0NZV83P5hOMc1ABVUIBfPXZbOqfT45HqEmLQi7K9U/slJgL2EZKHXeJ8xF8jsrLKn3gvQCquQrPPaLiiqvO5nmetFZ8+6m8OZMzax92WHg==
"###;

const FREEDESKTOP_REPO_SPEC: &str = r###"
[Flatpak Repo]
Title=Freedesktop SDK
Url=https://cache.sdk.freedesktop.org/releases/
Homepage=https://gitlab.com/freedesktop-sdk/
Comment=Repository for Freedesktop SDK
Description=Repository for Freedesktop SDK
GPGKey=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
"###;

#[derive(Debug, Serialize, Deserialize)]
pub enum FlatpakManifestFormat {
    JSON,
    YAML,
}
impl Default for FlatpakManifestFormat {
    fn default() -> Self {
        FlatpakManifestFormat::YAML
    }
}

// See `man flatpak-manifest` for the flatpak manifest specs.
#[derive(Deserialize, Serialize, Debug, Default)]
#[serde(rename_all = "kebab-case")]
#[serde(default)]
pub struct FlatpakManifest {
    #[serde(skip_serializing)]
    pub format: FlatpakManifestFormat,

    // Name of the application.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub app_name: String,

    // A string defining the application id.
    // Both names (app-id and id) are accepted.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub app_id: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub id: String,

    // The branch to use when exporting the application.
    // If this is unset the defaults come from the default-branch option.
    //
    // This key overrides both the default-branch key, and the --default-branch commandline option.
    // Unless you need a very specific branchname (like for a runtime or an extension) it is recommended
    // to use the default-branch key instead, because you can then override the default using
    // --default-branch when building for instance a test build.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub branch: String,

    // The default branch to use when exporting the application. Defaults to master.
    // This key can be overridden by the --default-branch commandline option.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub default_branch: String,

    // The collection ID of the repository, defaults to being unset.
    // Setting a globally unique collection ID allows the apps in the
    // repository to be shared over peer to peer systems without needing further configuration.
    // If building in an existing repository, the collection ID must match the existing
    // configured collection ID for that repository.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub collection_id: String,

    // The name of the runtime that the application uses.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub runtime: String,

    // The version of the runtime that the application uses, defaults to master.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub runtime_version: String,

    // The name of the development runtime that the application builds with.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub sdk: String,

    // The name of the development extensions that the application requires to build.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub sdk_extensions: Vec<String>,

    // Initialize the (otherwise empty) writable /var in the build with a copy of this runtime.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub var: String,

    // Use this file as the base metadata file when finishing.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub metadata: String,

    // Build a new runtime instead of an application.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub build_runtime: Option<bool>,

    // Whether the manifest describes an extension to be used by other manifests.
    // Extensions can be used to bundle programming langages and their associated
    // tools, for example.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub build_extension: Option<bool>,

    // Start with the files from the specified application.
    // This can be used to create applications that extend another application.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub base: String,

    // Use this specific version of the application specified in base.
    // If unspecified, this uses the value specified in branch
    #[serde(skip_serializing_if = "String::is_empty")]
    pub base_version: String,

    // Install these extra extensions from the base application when
    // initializing the application directory.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub base_extensions: Vec<String>,

    // Inherit these extra extensions points from the base application or
    // sdk when finishing the build.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub inherit_extensions: Vec<String>,

    // Inherit these extra extensions points from the base application or sdk
    // when finishing the build, but do not inherit them into the platform.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub inherit_sdk_extensions: Vec<String>,

    // Inherit these extra extensions points from the base application or sdk when finishing the build,
    // but do not inherit them into the platform.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub build_options: Option<FlatpakBuildOptions>,

    // The name of the command that the flatpak should run on execution.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub command: String,

    // Add these tags to the metadata file.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,

    // This is a dictionary of extension objects.
    // The key is the name of the extension.
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub add_extensions: BTreeMap<String, FlatpakExtension>,

    // This is a dictionary of extension objects similar to add-extensions.
    // The main difference is that the extensions are added early and are
    // available for use during the build.
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub add_build_extensions: BTreeMap<String, FlatpakExtension>,

    // An array of file patterns that should be removed at the end.
    // Patterns starting with / are taken to be full pathnames (without the /app prefix),
    // otherwise they just match the basename.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub cleanup: Vec<String>,

    // An array of commandlines that are run during the cleanup phase.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub cleanup_commands: Vec<String>,

    // Extra files to clean up in the platform.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub cleanup_platform: Vec<String>,

    // An array of commandlines that are run during the cleanup phase of the platform.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub cleanup_platform_commands: Vec<String>,

    // An array of commandlines that are run after importing the base platform,
    // but before applying the new files from the sdk. This is a good place to e.g. delete
    // things from the base that may conflict with the files added in the sdk.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub prepare_platform_commands: Vec<String>,

    // An array of arguments passed to the flatpak build-finish command.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub finish_args: Vec<String>,

    // Any desktop file with this name will be renamed to a name
    // based on id during the cleanup phase.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub rename_desktop_file: String,

    // Any appdata file with this name will be renamed to a name based
    // on id during the cleanup phase.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub rename_appdata_file: String,

    // Any icon with this name will be renamed to a name based on id during
    // the cleanup phase. Note that this is the icon name, not the full filenames,
    // so it should not include a filename extension.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub rename_icon: String,

    // Replace the appdata project-license field with this string.
    // This is useful as the upstream license is typically only about
    // the application itself, whereas the bundled app can contain other
    // licenses too.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub appdata_license: String,

    // If rename-icon is set, keep a copy of the old icon file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub copy_icon: Option<bool>,

    // This string will be prefixed to the Name key in the main application desktop file.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub desktop_file_name_prefix: String,

    // This string will be suffixed to the Name key in the main application desktop file.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub desktop_file_name_suffix: String,

    // An array of strings specifying the modules to be built in order.
    // String members in the array are interpreted as the name of a separate
    // json or yaml file that contains a module. See below for details.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub modules: Vec<FlatpakModule>,
}
impl FlatpakManifest {
    pub fn load_from_file(path: String) -> Option<FlatpakManifest> {
        let file_path = path::Path::new(&path);
        if !file_path.is_file() {
            log::error!("{} is not a file.", path);
            return None;
        }

        if FlatpakManifest::file_path_matches(&file_path.to_str().unwrap()) {
            let manifest_content = match fs::read_to_string(file_path) {
                Ok(content) => content,
                Err(e) => {
                    log::error!("Could not read manifest file {}: {}.", path, e);
                    return None;
                }
            };
            log::info!("Parsing Flatpak manifest file {}", &path);
            let mut manifest = match FlatpakManifest::parse(&path, &manifest_content) {
                Ok(m) => m,
                Err(e) => {
                    log::warn!("Failed to parse Flatpak manifest at {}: {}", path, e);
                    return None;
                }
            };
            return Some(manifest);
        } else {
            log::debug!("{} is not a Flatpak manifest.", path);
            return None;
        }
    }

    pub fn file_extension_matches(path: &str) -> bool {
        if path.to_lowercase().ends_with("yml") || path.to_lowercase().ends_with("yaml") {
            return true;
        }
        if path.to_lowercase().ends_with("json") {
            return true;
        }
        return false;
    }

    pub fn file_path_matches(path: &str) -> bool {
        lazy_static! {
            static ref REVERSE_DNS_FILENAME_REGEX: Regex = Regex::new(
                r"[a-z][a-z][a-z]*\.[a-z][0-9a-zA-Z_\-]+\.[a-z][0-9a-zA-Z_\-]+(\.[a-z][0-9a-zA-Z_\-]+)*\.(json|yaml|yml)$"
            ).unwrap();
        }
        REVERSE_DNS_FILENAME_REGEX.is_match(&path.to_lowercase())
    }

    pub fn parse(manifest_path: &str, manifest_content: &str) -> Result<FlatpakManifest, String> {
        let mut flatpak_manifest: FlatpakManifest = FlatpakManifest::default();

        if manifest_path.to_lowercase().ends_with("yaml") || manifest_path.to_lowercase().ends_with("yml") {
            flatpak_manifest = match serde_yaml::from_str(&manifest_content) {
                Ok(m) => m,
                Err(e) => {
                    return Err(format!("Failed to parse the Flatpak manifest: {}.", e));
                }
            };
            flatpak_manifest.format = FlatpakManifestFormat::YAML;
        } else if manifest_path.to_lowercase().ends_with("json") {
            let json_content_without_comments = crate::utils::remove_comments_from_json(manifest_content);
            flatpak_manifest = match serde_json::from_str(&json_content_without_comments) {
                Ok(m) => m,
                Err(e) => {
                    return Err(format!("Failed to parse the Flatpak manifest: {}.", e));
                }
            };
            flatpak_manifest.format = FlatpakManifestFormat::JSON;
        }

        // From https://docs.flatpak.org/en/latest/manifests.html#basic-properties:
        // Each manifest file should specify basic information about the application that is to be built,
        // including the app-id, runtime, runtime-version, sdk and command parameters.
        if flatpak_manifest.app_id.is_empty() && flatpak_manifest.id.is_empty() {
            return Err(
                "Required top-level field id (or app-id) is missing from Flatpak manifest.".to_string(),
            );
        }
        if flatpak_manifest.runtime.is_empty() {
            return Err("Required top-level field runtime is missing from Flatpak manifest.".to_string());
        }
        if flatpak_manifest.runtime_version.is_empty() {
            return Err(
                "Required top-level field runtime-version is missing from Flatpak manifest.".to_string(),
            );
        }
        if flatpak_manifest.sdk.is_empty() {
            return Err("Required top-level field sdk is missing from Flatpak manifest.".to_string());
        }
        if flatpak_manifest.command.is_empty() {
            return Err("Required top-level field command is missing from Flatpak manifest.".to_string());
        }

        Ok(flatpak_manifest)
    }

    pub fn dump(&self) -> Result<String, String> {
        if let FlatpakManifestFormat::JSON = self.format {
            return match serde_json::to_string_pretty(&self) {
                Ok(d) => Ok(d),
                Err(e) => return Err(format!("Failed to dump the Flatpak manifest: {}.", e)),
            };
        }

        if let FlatpakManifestFormat::YAML = self.format {
            return match serde_yaml::to_string(&self) {
                Ok(d) => Ok(d),
                Err(e) => return Err(format!("Failed to dump the Flatpak manifest: {}.", e)),
            };
        }

        Err(format!("Invalid format for Flatpak manifest."))
    }

    pub fn get_all_module_urls(&self) -> Vec<String> {
        let mut all_urls = vec![];
        for module in &self.modules {
            let module: &FlatpakModuleDescription = match module {
                FlatpakModule::Path(_) => continue,
                FlatpakModule::Description(m) => &m,
            };
            all_urls.append(&mut module.get_all_urls());
        }
        all_urls
    }

    pub fn get_main_module_url(&self) -> Option<String> {
        let main_module = match self.modules.last() {
            Some(m) => m,
            None => return None,
        };
        let main_module: &FlatpakModuleDescription = match main_module {
            FlatpakModule::Path(_) => return None,
            FlatpakModule::Description(m) => m,
        };
        return main_module.get_main_url();
    }

    pub fn get_max_depth(&self) -> i32 {
        let mut max_depth: i32 = 1;
        for module in &self.modules {
            if let FlatpakModule::Description(module_description) = module {
                let module_depth = module_description.get_max_depth();
                if module_depth > max_depth {
                    max_depth = module_depth;
                }
            }
        }
        return max_depth;
    }
}

// Each module item can be either a path to a module description file,
// or an inline module description.
#[derive(Debug, Deserialize, Serialize, Hash)]
#[serde(rename_all = "kebab-case")]
#[serde(untagged)]
pub enum FlatpakModule {
    Path(String),
    Description(FlatpakModuleDescription),
}
impl FlatpakModule {
    pub fn get_all_repos_urls(&self) -> Vec<String> {
        if let FlatpakModule::Description(module_description) = self {
            return module_description.get_all_urls();
        } else {
            return vec![];
        }
    }
    pub fn is_patched(&self) -> bool {
        match self {
            FlatpakModule::Path(_) => return false,
            FlatpakModule::Description(d) => {
                for source in &d.sources {
                    if let FlatpakSource::Description(sd) = source {
                        if let Some(t) = &sd.r#type {
                            if t == "patch" {
                                return true;
                            }
                        }
                    }
                }
            }
        }
        return false;
    }
}

// Each module specifies a source that has to be separately built and installed.
// It contains the build options and a list of sources to download and extract before
// building.
//
// Modules can be nested, in order to turn related modules on and off with a single key.
#[derive(Debug, Default, Deserialize, Serialize, Hash)]
#[serde(rename_all = "kebab-case")]
#[serde(default)]
pub struct FlatpakModuleDescription {
    // The name of the module, used in e.g. build logs. The name is also
    // used for constructing filenames and commandline arguments,
    // therefore using spaces or '/' in this string is a bad idea.
    pub name: String,

    // If true, skip this module
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disabled: Option<bool>,

    // An array of objects defining sources that will be downloaded and extracted in order.
    // String members in the array are interpreted as the name of a separate
    // json or yaml file that contains sources. See below for details.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub sources: Vec<FlatpakSource>,

    // An array of options that will be passed to configure
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub config_opts: Vec<String>,

    // An array of arguments that will be passed to make
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub make_args: Vec<String>,

    // An array of arguments that will be passed to make install
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub make_install_args: Vec<String>,

    // If true, remove the configure script before starting build
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rm_configure: Option<bool>,

    // Ignore the existence of an autogen script
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_autogen: Option<bool>,

    // Don't call make with arguments to build in parallel
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_parallel_make: Option<bool>,

    // Name of the rule passed to make for the install phase, default is install
    #[serde(skip_serializing_if = "String::is_empty")]
    pub install_rule: String,

    // Don't run the make install (or equivalent) stage
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_make_install: Option<bool>,

    // Don't fix up the *.py[oc] header timestamps for ostree use.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_python_timestamp_fix: Option<bool>,

    // Use cmake instead of configure (deprecated: use buildsystem instead)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cmake: Option<bool>,

    // Build system to use: autotools, cmake, cmake-ninja, meson, simple, qmake
    #[serde(skip_serializing_if = "String::is_empty")]
    pub buildsystem: String,

    // Use a build directory that is separate from the source directory
    #[serde(skip_serializing_if = "Option::is_none")]
    pub builddir: Option<bool>,

    // Build inside this subdirectory of the extracted sources
    #[serde(skip_serializing_if = "String::is_empty")]
    pub subdir: String,

    // A build options object that can override global options
    #[serde(skip_serializing_if = "Option::is_none")]
    pub build_options: Option<FlatpakBuildOptions>,

    // An array of commands to run during build (between make and make install if those are used).
    // This is primarily useful when using the "simple" buildsystem.
    // Each command is run in /bin/sh -c, so it can use standard POSIX shell syntax such as piping output.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub build_commands: Vec<String>,

    // An array of shell commands that are run after the install phase.
    // Can for example clean up the install dir, or install extra files.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub post_install: Vec<String>,

    // An array of file patterns that should be removed at the end.
    // Patterns starting with / are taken to be full pathnames (without the /app prefix), otherwise
    // they just match the basename. Note that any patterns will only match
    // files installed by this module.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub cleanup: Vec<String>,

    // The way the builder works is that files in the install directory are hard-links to the cached files,
    // so you're not allowed to modify them in-place. If you list a file in this then the hardlink
    // will be broken and you can modify it. This is a workaround, ideally installing files should
    // replace files, not modify existing ones.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub ensure_writable: Vec<String>,

    // If non-empty, only build the module on the arches listed.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub only_arches: Vec<String>,

    // Don't build on any of the arches listed.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub skip_arches: Vec<String>,

    // Extra files to clean up in the platform.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub cleanup_platform: Vec<String>,

    // If true this will run the tests after installing.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run_tests: Option<bool>,

    // The target to build when running the tests. Defaults to "check" for make and "test" for ninja.
    // Set to empty to disable.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub test_rule: String,

    // Array of commands to run during the tests.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub test_commands: Vec<String>,

    // An array of objects specifying nested modules to be built before this one.
    // String members in the array are interpreted as names of a separate json or
    // yaml file that contains a module.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub modules: Vec<FlatpakModule>,
}
impl FlatpakModuleDescription {
    pub fn load_from_file(path: String) -> Option<FlatpakModuleDescription> {
        let file_path = path::Path::new(&path);
        if !file_path.is_file() {
            log::error!("{} is not a file.", path);
            return None;
        }

        if FlatpakModuleDescription::file_path_matches(&file_path.to_str().unwrap()) {
            let module_content = match fs::read_to_string(file_path) {
                Ok(content) => content,
                Err(e) => {
                    log::error!("Could not read manifest file {}: {}.", path, e);
                    return None;
                }
            };
            log::info!("Parsing Flatpak module file {}", &path);
            let mut module = match FlatpakModuleDescription::parse(&path, &module_content) {
                Ok(m) => m,
                Err(e) => {
                    log::warn!("Failed to parse Flatpak module at {}: {}", path, e);
                    return None;
                }
            };
            return Some(module);
        } else {
            log::debug!("{} is not a Flatpak module.", path);
            return None;
        }
    }

    pub fn parse(module_path: &str, module_content: &str) -> Result<FlatpakModuleDescription, String> {
        let mut flatpak_module: FlatpakModuleDescription = FlatpakModuleDescription::default();

        if module_path.to_lowercase().ends_with("yaml") || module_path.to_lowercase().ends_with("yml") {
            flatpak_module = match serde_yaml::from_str(&module_content) {
                Ok(m) => m,
                Err(e) => {
                    return Err(format!("Failed to parse the Flatpak manifest: {}.", e));
                }
            };
        } else if module_path.to_lowercase().ends_with("json") {
            let json_content_without_comments = crate::utils::remove_comments_from_json(module_content);
            flatpak_module = match serde_json::from_str(&json_content_without_comments) {
                Ok(m) => m,
                Err(e) => {
                    return Err(format!("Failed to parse the Flatpak manifest: {}.", e));
                }
            };
        }

        if flatpak_module.name.is_empty() {
            return Err("Required top-level field name is missing from Flatpak module.".to_string());
        }
        if flatpak_module.sources.is_empty() {
            return Err("Required sources were not found in Flatpak module.".to_string());
        }

        Ok(flatpak_module)
    }

    pub fn file_path_matches(path: &str) -> bool {
        return FlatpakManifest::file_extension_matches(path);
    }

    pub fn get_hash(&self) -> u64 {
        let mut s = DefaultHasher::new();
        self.hash(&mut s);
        s.finish()
    }

    pub fn get_all_urls(&self) -> Vec<String> {
        let mut all_urls = vec![];
        for module in &self.modules {
            if let FlatpakModule::Description(module_description) = module {
                all_urls.append(&mut module_description.get_all_urls());
            }
        }
        for source in &self.sources {
            for url in source.get_all_urls() {
                all_urls.push(url);
            }
        }
        all_urls
    }

    pub fn get_max_depth(&self) -> i32 {
        let mut max_depth: i32 = 0;
        for module in &self.modules {
            if let FlatpakModule::Description(module_description) = module {
                let module_depth = module_description.get_max_depth();
                if module_depth > max_depth {
                    max_depth = module_depth;
                }
            }
        }
        return max_depth + 1;
    }

    pub fn get_main_url(&self) -> Option<String> {
        if self.sources.len() < 1 {
            return None;
        }

        // Here we assume that the first source is the actual project, and
        // anything after is a patch or an additional file.
        let main_module_source = self.sources.first().unwrap();

        let main_module_source_url: Option<String> = main_module_source.get_url();

        match &main_module_source_url {
            Some(s) => Some(s.to_string()),
            None => None,
        }
    }
}

lazy_static! {
    static ref SOURCE_TYPES: Vec<String> = vec![
        "archive".to_string(),
        "git".to_string(),
        "bzr".to_string(),
        "svn".to_string(),
        "dir".to_string(),
        "file".to_string(),
        "script".to_string(),
        "shell".to_string(),
        "patch".to_string(),
        "extra-data".to_string(),
    ];
}

pub const DEFAULT_SOURCE_TYPE: &str = "archive";

// The sources are a list pointer to the source code that needs to be extracted into
// the build directory before the build starts.
// They can be of several types, distinguished by the type property.
//
// Additionally, the sources list can contain a plain string, which is interpreted as the name
// of a separate json or yaml file that is read and inserted at this
// point. The file can contain a single source, or an array of sources.
#[derive(Debug, Deserialize, Serialize, Hash)]
#[serde(rename_all = "kebab-case")]
#[serde(untagged)]
pub enum FlatpakSource {
    Path(String),
    Description(FlatpakSourceDescription),
}
impl FlatpakSource {
    pub fn get_url(&self) -> Option<String> {
        None
    }
    pub fn get_all_urls(&self) -> Vec<String> {
        let mut response: Vec<String> = vec![];
        let source_description = match self {
            FlatpakSource::Path(_) => return response,
            FlatpakSource::Description(sd) => sd,
        };
        if let Some(url) = &source_description.url {
            response.push(url.to_string());
        }
        if let Some(urls) = &source_description.mirror_urls {
            for url in urls {
                response.push(url.to_string());
            }
        }
        return response;
    }
    pub fn get_type_name(&self) -> String {
        return match self {
            FlatpakSource::Path(_) => "path".to_string(),
            FlatpakSource::Description(d) => {
                if let Some(t) = &d.r#type {
                    return t.to_string();
                }
                return DEFAULT_SOURCE_TYPE.to_string();
            }
        };
    }
    pub fn type_is_valid(&self) -> bool {
        return match self {
            FlatpakSource::Path(_) => true,
            FlatpakSource::Description(d) => {
                if let Some(t) = &d.r#type {
                    return SOURCE_TYPES.contains(&t);
                }
                return false;
            }
        };
    }
    pub fn type_is_empty(&self) -> bool {
        return match self {
            FlatpakSource::Path(_) => false,
            FlatpakSource::Description(d) => d.r#type.is_none(),
        };
    }
}

#[derive(Debug, Default, Deserialize, Serialize, Hash)]
#[serde(rename_all = "kebab-case")]
pub struct FlatpakSourceDescription {
    // Defines the type of the source description. This field is optional.
    // TODO is there a default or can the source type be infered?
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,

    // An array of shell commands.
    // types: script, shell
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commands: Option<Vec<String>>,

    // Filename to use inside the source dir.
    // types: script, archive, file
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dest_filename: Option<String>,

    // The name to use for the downloaded extra data
    // types: extra-data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filename: Option<String>,

    // The url to the resource.
    // types: extra-data, svn, bzr, git, archive, file
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    // A list of alternative urls that are used if the main url fails.
    // types: archive, file
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mirror_urls: Option<Vec<String>>,

    // The md5 checksum of the file, verified after download
    // Note that md5 is no longer considered a safe checksum, we recommend you use at least sha256.
    // types: archive, file
    #[serde(skip_serializing_if = "Option::is_none")]
    pub md5: Option<String>,

    // The sha1 checksum of the file, verified after download
    // Note that sha1 is no longer considered a safe checksum, we recommend you use at least sha256.
    // types: archive, file
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha1: Option<String>,

    // The sha256 of the resource.
    // types: extra-data, archive, file
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha256: Option<String>,

    // The sha512 checksum of the file, verified after download
    // types: archive, file
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha512: Option<String>,

    // The size of the extra data in bytes.
    // types: extra-data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<i64>,

    // Whether to initialise the repository as a git repository.
    // types: archive
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git_init: Option<bool>,

    // The extra installed size this adds to the app (optional).
    // types: extra-data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub installed_size: Option<String>,

    // A specific revision number to use
    // types: svn, bzr
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revision: Option<String>,

    // The branch to use from the git repository
    // types: git
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,

    // The type of archive if it cannot be guessed from the path.
    // Possible values are "rpm", "tar", "tar-gzip", "tar-compress", "tar-bzip2", "tar-lzip", "tar-lzma", "tar-lzop", "tar-xz", "zip" and "7z".
    // types: archive
    #[serde(skip_serializing_if = "Option::is_none")]
    pub archive_type: Option<String>,

    // The commit to use from the git repository.
    // If branch is also specified, then it is verified that the branch/tag is at this specific commit.
    // This is a readable way to document that you're using a particular tag, but verify that it does not change.
    // types: git
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit: Option<String>,

    // The path to associated with the resource.
    // types: git, archive, dir, patch, file
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,

    // An list of paths to a patch files that will be applied in the source dir, in order
    // types: patch
    #[serde(skip_serializing_if = "Option::is_none")]
    pub paths: Option<Vec<String>>,

    // Whether to use "git apply" rather than "patch" to apply the patch, required when the patch file contains binary diffs.
    // types: patch
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_git: Option<bool>,

    // Whether to use "git am" rather than "patch" to apply the patch, required when the patch file contains binary diffs.
    // You cannot use this at the same time as use-git.
    // types: patch
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_git_am: Option<bool>,

    // Extra options to pass to the patch command.
    // types: patch
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<Vec<String>>,

    // Don't use transfer.fsckObjects=1 to mirror git repository. This may be needed for some (broken) repositories.
    // types: git
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disable_fsckobjects: Option<bool>,

    // Don't optimize by making a shallow clone when downloading the git repo.
    // types: git
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disable_shallow_clone: Option<bool>,

    // Don't checkout the git submodules when cloning the repository.
    // types: git
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disable_submodules: Option<bool>,

    // The number of initial pathname components to strip.
    // types: archive, patch
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strip_components: Option<i64>,

    // Source files to ignore in the directory.
    // types: dir
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skip: Option<Vec<String>>,

    // If non-empty, only build the module on the arches listed.
    // types: all
    #[serde(skip_serializing_if = "Option::is_none")]
    pub only_arches: Option<Vec<String>>,

    // Don't build on any of the arches listed.
    // types: all
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skip_arches: Option<Vec<String>>,

    // Directory inside the source dir where this source will be extracted.
    // types: all
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dest: Option<String>,
}

// Extension define extension points in the app/runtime that can be implemented by extensions,
// supplying extra files which are available during runtime..
//
// Additionally the standard flatpak extension properties are supported, and put
// directly into the metadata file: autodelete, no-autodownload, subdirectories,
// add-ld-path, download-if, enable-if, merge-dirs, subdirectory-suffix, locale-subset,
// version, versions. See the flatpak metadata documentation for more information on these.
#[derive(Deserialize, Serialize, Default, Debug)]
#[serde(rename_all = "kebab-case")]
#[serde(default)]
pub struct FlatpakExtension {
    // The directory where the extension is mounted. If the extension point is for an application,
    // this path is relative to /app, otherwise it is relative to /usr.
    pub extension_directory: String,

    // If this is true, then the data created in the extension directory is omitted from the result,
    // and instead packaged in a separate extension.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bundle: Option<bool>,

    // If this is true, the extension is removed during when finishing.
    // This is only interesting for extensions in the add-build-extensions property.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remove_after_build: Option<bool>,

    // Whether to automatically delete extensions matching this extension point
    // when deleting a 'related' application or runtime.
    pub autodelete: Option<bool>,

    // Whether to automatically download extensions matching this extension point
    // when updating or installing a 'related' application or runtime.
    pub no_autodownload: Option<bool>,

    // If this key is set to true, then flatpak will look for extensions whose name is a
    // prefix of the extension point name, and mount them at the corresponding
    // name below the subdirectory.
    pub subdirectories: Option<bool>,

    // A path relative to the extension point directory that will be appended to LD_LIBRARY_PATH.
    pub add_ld_path: Option<String>,

    // A list of conditions, separated by semi-colons, that must be true for the extension to be auto-downloaded.
    // These are the supported conditions:
    //  active-gl-driver
    //     Is true if the name of the active GL driver matches the extension point basename.
    //  active-gtk-theme
    //     Is true if the name of the current GTK theme (via org.gnome.desktop.interface GSetting)
    //     matches the extension point basename.
    //  have-intel-gpu
    //     Is true if the i915 kernel module is loaded.
    //  on-xdg-desktop-*
    //     Is true if the suffix (case-insensitively) is in the XDG_CURRENT_DESKTOP env var.
    //     For example on-xdg-desktop-GNOME-classic.
    pub download_if: Option<String>,

    // A list of conditions, separated by semi-colons, that must be true for the extension to be
    // enabled. See download_if for available conditions.
    pub enable_if: Option<String>,

    // A list of relative paths of directories below the extension point directory that will be merged.
    pub merge_dirs: Option<String>,

    // A suffix that gets appended to the directory name.
    // This is very useful when the extension point naming scheme is "reversed".
    // For example, an extension point for GTK+ themes would be /usr/share/themes/$NAME/gtk-3.0,
    // which could be achieved using subdirectory-suffix=gtk-3.0.
    pub subdirectory_suffix: Option<String>,

    // If set, then the extensions are partially downloaded by default, based on the currently
    // configured locales. This means that the extension contents should be
    // a set of directories with the language code as name.
    pub locale_subset: Option<bool>,

    // The branch to use when looking for the extension.
    // If this is not specified, it defaults to the branch of the application or
    // runtime that the extension point is for.
    pub version: Option<String>,

    // The branches to use when looking for the extension.
    // If this is not specified, it defaults to the branch of the application or
    // runtime that the extension point is for.
    pub versions: Option<String>,
}

// Build options specify the build environment of a module,
// and can be specified globally as well as per-module.
// Options can also be specified on a per-architecture basis using the arch property.
#[derive(Deserialize, Serialize, Debug, Default, Hash)]
#[serde(rename_all = "kebab-case")]
#[serde(default)]
pub struct FlatpakBuildOptions {
    // This is set in the environment variable CFLAGS during the build.
    // Multiple specifications of this (in e.g. per-arch area) are concatenated, separated by spaces.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub cflags: String,

    // If this is true, clear cflags from previous build options before adding it from these options.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cflags_override: Option<bool>,

    // This is set in the environment variable CPPFLAGS during the build.
    // Multiple specifications of this (in e.g. per-arch area) are concatenated, separated by spaces.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub cppflags: String,

    // If this is true, clear cppflags from previous build options before adding it from these options.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cppflags_override: Option<bool>,

    // This is set in the environment variable CXXFLAGS during the build.
    // Multiple specifications of this (in e.g. per-arch area) are concatenated, separated by spaces.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub cxxflags: String,

    // If this is true, clear cxxflags from previous build options before adding it from these options.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cxxflags_override: Option<bool>,

    // This is set in the environment variable LDFLAGS during the build.
    // Multiple specifications of this (in e.g. per-arch area) are concatenated,
    // separated by spaces.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub ldflags: String,

    // If this is true, clear ldflags from previous build options before adding it from these options.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ldflags_override: Option<bool>,

    // The build prefix for the modules (defaults to /app for applications and /usr for runtimes).
    #[serde(skip_serializing_if = "String::is_empty")]
    pub prefix: String,

    // The build libdir for the modules (defaults to /app/lib for applications and /usr/lib for runtimes).
    #[serde(skip_serializing_if = "String::is_empty")]
    pub libdir: String,

    // This will get appended to PATH in the build environment (with an leading colon if needed).
    #[serde(skip_serializing_if = "String::is_empty")]
    pub append_path: String,

    // This will get prepended to PATH in the build environment (with an trailing colon if needed).
    #[serde(skip_serializing_if = "String::is_empty")]
    pub prepend_path: String,

    // This will get appended to LD_LIBRARY_PATH in the build environment (with an leading colon if needed).
    #[serde(skip_serializing_if = "String::is_empty")]
    pub append_ld_library_path: String,

    // This will get prepended to LD_LIBRARY_PATH in the build environment (with an trailing colon if needed).
    #[serde(skip_serializing_if = "String::is_empty")]
    pub prepend_ld_library_path: String,

    // This will get appended to PKG_CONFIG_PATH in the build environment (with an leading colon if needed).
    #[serde(skip_serializing_if = "String::is_empty")]
    pub append_pkg_config_path: String,

    // This will get prepended to PKG_CONFIG_PATH in the build environment (with an trailing colon if needed).
    #[serde(skip_serializing_if = "String::is_empty")]
    pub prepend_pkg_config_path: String,

    // This is a dictionary defining environment variables to be set during the build.
    // Elements in this override the properties that set the environment, like
    // cflags and ldflags. Keys with a null value unset the corresponding variable.
    pub env: BTreeMap<String, String>,

    // This is an array containing extra options to pass to flatpak build.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub build_args: Vec<String>,

    // Similar to build-args but affects the tests, not the normal build.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub test_args: Vec<String>,

    // This is an array containing extra options to pass to configure.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub config_opts: Vec<String>,

    // An array of extra arguments that will be passed to make
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub make_args: Vec<String>,

    // An array of extra arguments that will be passed to make install
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub make_install_args: Vec<String>,

    // If this is true (the default is false) then all ELF files will be stripped after install.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strip: Option<bool>,

    // By default (if strip is not true) flatpak-builder extracts all debug info in ELF files to a
    // separate files and puts this in an extension. If you want to disable this, set no-debuginfo
    // to true.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_debuginfo: Option<bool>,

    // By default when extracting debuginfo we compress the debug sections.
    // If you want to disable this, set no-debuginfo-compression to true.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_debuginfo_compression: Option<bool>,

    // This is a dictionary defining for each arch a separate build options object that override the main one.
    pub arch: BTreeMap<String, FlatpakBuildOptions>,
}

/// Setup the system
pub fn setup() -> Result<String, String> {
    let child = Command::new("flatpak")
        .arg("remote-add")
        .arg("--if-not-exists")
        .arg("--user")
        .arg("flathub")
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let output = match child.wait_with_output() {
        Ok(o) => o,
        Err(e) => return Err(e.to_string()),
    };
    if !output.status.success() {
        return Ok("it went ok".to_string());
    }
    Ok(String::from("lol"))
}

pub fn is_setup() -> bool {
    let child = Command::new("flatpak")
        .arg("remote-list")
        .arg("--user")
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let output = match child.wait_with_output() {
        Ok(o) => o,
        Err(e) => return false,
    };
    if !output.status.success() {
        return false;
    }
    let stdout = match str::from_utf8(&output.stdout) {
        Ok(v) => v,
        Err(e) => {
            log::warn!("Invalid UTF-8 sequence printed by `flatpak remote-list`.");
            return false;
        }
    };
    return true;
}

pub fn run_build() -> Result<String, String> {
    Ok(String::from("lol"))
}

pub fn run_command(command: &str) -> Result<String, String> {
    let flatpak_build_dir = path::Path::new(DEFAULT_FLATPAK_OUTPUT_DIR);
    if !flatpak_build_dir.is_dir() {
        return Err("Looks like this workspace was not built. Run `fpm make` first.".to_string());
    }

    let child = Command::new("flatpak-builder")
        .arg("--run")
        .arg(DEFAULT_FLATPAK_OUTPUT_DIR)
        .arg(&"CHANGE THIS PATH")
        .arg(command)
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let output = match child.wait_with_output() {
        Ok(o) => o,
        Err(e) => return Err(e.to_string()),
    };
    if !output.status.success() {
        return Ok("it went ok".to_string());
    }
    Ok(String::from("lol"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_file_path_matches() {
        assert!(FlatpakManifest::file_path_matches("com.example.appName.yaml"));
        assert!(FlatpakManifest::file_path_matches("COM.EXAMPLE.APPNAME.YAML"));
        assert!(FlatpakManifest::file_path_matches(
            "io.github.user.repo.Devel.yaml"
        ));
        assert!(FlatpakManifest::file_path_matches(
            "/path/to/com.example.appName.yaml"
        ));
        assert!(FlatpakManifest::file_path_matches(
            "/path/to/com.example.appName.yml"
        ));
        assert!(FlatpakManifest::file_path_matches(
            "/path/to/com.example.department.product.yaml"
        ));
        assert!(FlatpakManifest::file_path_matches(
            "/path/to/com.example.department.name-of-product.yaml"
        ));
        assert!(!FlatpakManifest::file_path_matches(
            "/tmp/com.github.flathub.org.freedesktop.LinuxAudio.Plugins.WolfShaper/flathub.json"
        ));
        assert!(!FlatpakManifest::file_path_matches("Firefox-62.0.3.update.json"));
        assert!(!FlatpakManifest::file_path_matches("/path/to/file.yaml"));
        assert!(!FlatpakManifest::file_path_matches("/path/to/file.json"));
        assert!(!FlatpakManifest::file_path_matches("/path/to/___432423fdsf.json"));
        assert!(!FlatpakManifest::file_path_matches("/path/to/example.com.json"));
        assert!(!FlatpakManifest::file_path_matches("/path/to/example.com.json."));
        assert!(!FlatpakManifest::file_path_matches(""));
        assert!(!FlatpakManifest::file_path_matches("/////////////"));
    }

    #[test]
    #[should_panic]
    pub fn test_parse_invalid_yaml() {
        FlatpakManifest::parse("manifest.yaml", "----------------------------").unwrap();
    }

    #[test]
    pub fn test_parse_missing_fields() {
        assert!(FlatpakManifest::parse(
            "manifest.yaml",
            r###"
            runtime: org.gnome.Platform
            runtime-version: "3.36"
            sdk: org.gnome.Sdk
            command: fpm
            "###
        )
        .is_err());
    }

    #[test]
    pub fn test_parse() {
        match FlatpakManifest::parse(
            "manifest.yaml",
            r###"
            app-id: net.louib.fpm
            runtime: org.gnome.Platform
            runtime-version: "3.36"
            sdk: org.gnome.Sdk
            command: fpm
            tags: ["nightly"]
            modules:
              -
                name: "fpm"
                buildsystem: simple
                cleanup: [ "*" ]
                config-opts: []
                sources:
                  -
                    type: git
                    url: https://github.com/louib/fpm.git
                    branch: master
            "###,
        ) {
            Err(e) => panic!(e),
            Ok(manifest) => {
                assert_eq!(manifest.app_id, "net.louib.fpm");
            }
        }
    }

    #[test]
    pub fn test_parse_shared_modules() {
        match FlatpakManifest::parse(
            "manifest.yaml",
            r###"
            app-id: net.louib.fpm
            runtime: org.gnome.Platform
            runtime-version: "3.36"
            sdk: org.gnome.Sdk
            command: fpm
            tags: ["nightly"]
            modules:
              -
                name: "fpm"
                buildsystem: simple
                cleanup: [ "*" ]
                config-opts: []
                sources:
                  -
                    type: git
                    url: https://github.com/louib/fpm.git
                    branch: master
              -
                "shared-modules/linux-audio/lv2.json"
            "###,
        ) {
            Err(e) => panic!(e),
            Ok(manifest) => {
                assert_eq!(manifest.app_id, "net.louib.fpm");
            }
        }
    }

    #[test]
    pub fn test_parse_add_extensions() {
        match FlatpakManifest::parse(
            "manifest.yaml",
            r###"
            app-id: net.pcsx2.PCSX2
            runtime: org.freedesktop.Platform
            runtime-version: "19.08"
            sdk: org.freedesktop.Sdk
            command: PCSX2
            tags: ["nightly"]
            modules: []
            add-extensions:
                "org.freedesktop.Platform.Compat.i386":
                    directory: "lib/i386-linux-gnu"
                    version: "19.08"
                "org.freedesktop.Platform.Compat.i386.Debug":
                    directory: "lib/debug/lib/i386-linux-gnu"
                    version: "19.08"
                    no-autodownload: true
                "org.freedesktop.Platform.GL32":
                    directory: "lib/i386-linux-gnu/GL"
                    version: "1.4"
                    versions: "19.08;1.4"
                    subdirectories: true
                    no-autodownload: true
                    autodelete: false
                    add-ld-path: "lib"
                    merge-dirs: "vulkan/icd.d;glvnd/egl_vendor.d"
                    download-if: "active-gl-driver"
                    enable-if: "active-gl-driver"
            "###,
        ) {
            Err(e) => panic!(e),
            Ok(manifest) => {
                assert_eq!(manifest.app_id, "net.pcsx2.PCSX2");
                assert_eq!(manifest.add_extensions.len(), 3);
            }
        }
    }

    #[test]
    pub fn test_parse_string_source() {
        match FlatpakManifest::parse(
            "manifest.yaml",
            r###"
            app-id: net.louib.fpm
            runtime: org.gnome.Platform
            runtime-version: "3.36"
            sdk: org.gnome.Sdk
            command: fpm
            tags: ["nightly"]
            modules:
              -
                name: "fpm"
                buildsystem: simple
                cleanup: [ "*" ]
                config-opts: []
                sources:
                  -
                    "shared-modules/linux-audio/lv2.json"
            "###,
        ) {
            Err(e) => panic!(e),
            Ok(manifest) => {
                assert_eq!(manifest.app_id, "net.louib.fpm");
            }
        }
    }

    #[test]
    pub fn test_parse_source_without_type() {
        match FlatpakManifest::parse(
            "manifest.yaml",
            r###"
            app-id: net.louib.fpm
            runtime: org.gnome.Platform
            runtime-version: "3.36"
            sdk: org.gnome.Sdk
            command: fpm
            tags: ["nightly"]
            modules:
              -
                name: "gcc"
                buildsystem: simple
                cleanup: [ "*" ]
                config-opts: []
                sources:
                  -
                    url: "https://ftp.gnu.org/gnu/gcc/gcc-7.5.0/gcc-7.5.0.tar.xz"
                    sha256: "b81946e7f01f90528a1f7352ab08cc602b9ccc05d4e44da4bd501c5a189ee661"

            "###,
        ) {
            Err(e) => panic!(e),
            Ok(manifest) => {
                assert_eq!(manifest.app_id, "net.louib.fpm");
            }
        }
    }

    #[test]
    pub fn test_parse_build_options() {
        match FlatpakManifest::parse(
            "manifest.yaml",
            r###"
            app-id: net.louib.fpm
            runtime: org.gnome.Platform
            runtime-version: "3.36"
            sdk: org.gnome.Sdk
            command: fpm
            tags: ["nightly"]
            modules:
              -
                name: "fpm"
                buildsystem: simple
                cleanup: [ "*" ]
                build-options:
                   cflags: "-O2 -g"
                   cxxflags: "-O2 -g"
                   env:
                       V: "1"
                   arch:
                       x86_64:
                           cflags: "-O3 -g"
                config-opts: []
                sources:
                  -
                    "shared-modules/linux-audio/lv2.json"
            "###,
        ) {
            Err(e) => panic!(e),
            Ok(manifest) => {
                assert_eq!(manifest.app_id, "net.louib.fpm");
            }
        }
    }

    #[test]
    pub fn test_parse_script_source() {
        match FlatpakManifest::parse(
            "manifest.yaml",
            r###"
            app-id: net.louib.fpm
            runtime: org.gnome.Platform
            runtime-version: "3.36"
            sdk: org.gnome.Sdk
            command: fpm
            tags: ["nightly"]
            modules:
              -
                name: "fpm"
                buildsystem: simple
                cleanup: [ "*" ]
                config-opts: []
                sources:
                  -
                    url: "https://ftp.gnu.org/gnu/gcc/gcc-7.5.0/gcc-7.5.0.tar.xz"
                    sha256: "b81946e7f01f90528a1f7352ab08cc602b9ccc05d4e44da4bd501c5a189ee661"
                  -
                    type: "shell"
                    commands:
                      -
                        sed -i -e 's/\${\${NAME}_BIN}-NOTFOUND/\${NAME}_BIN-NOTFOUND/' cpp/CMakeLists.txt
            "###,
        ) {
            Err(e) => panic!(e),
            Ok(manifest) => {
                assert_eq!(manifest.app_id, "net.louib.fpm");
            }
        }
    }

    #[test]
    pub fn test_parse_json() {
        match FlatpakManifest::parse(
            "manifest.json",
            r###"
            {
                "app-id": "org.gnome.SoundJuicer",
                "runtime": "org.gnome.Platform",
                "runtime-version": "master",
                "sdk": "org.gnome.Sdk",
                "command": "sound-juicer",
                "tags": [ "nightly" ],
                "desktop-file-name-suffix": " ☢️",
                "finish-args": [
                    "--talk-name=org.gtk.vfs", "--talk-name=org.gtk.vfs.*",
                    "--env=GST_PLUGIN_PATH=/app/lib/codecs/lib/gstreamer-1.0"
                ],
                "cleanup": [ "/include", "/share/bash-completion" ],
                "modules": [
                    {
                        "name": "cdparanoia",
                        "buildsystem": "simple",
                        "build-commands": [
                            "cp /usr/share/automake-*/config.{sub,guess} .",
                            "./configure --prefix=/app",
                            "make all slib",
                            "make install"
                        ],
                        "sources": [
                            {
                                "type": "archive",
                                "url": "http://downloads.xiph.org/releases/cdparanoia/cdparanoia-III-10.2.src.tgz",
                                "sha256": "005db45ef4ee017f5c32ec124f913a0546e77014266c6a1c50df902a55fe64df"
                            },
                            {
                                "type": "patch",
                                "path": "cdparanoia-use-proper-gnu-config-files.patch"
                            }
                        ]
                    },
                    {
                        "name": "gst-plugins-base",
                        "buildsystem": "meson",
                        "config-opts": [
                            "--prefix=/app",
                            "-Dauto_features=disabled",
                            "-Dcdparanoia=enabled"
                        ],
                        "cleanup": [ "*.la", "/share/gtk-doc" ],
                        "sources": [
                            {
                                "type": "git",
                                "url": "https://gitlab.freedesktop.org/gstreamer/gst-plugins-base.git",
                                "branch" : "1.16.2",
                                "commit" : "9d3581b2e6f12f0b7e790d1ebb63b90cf5b1ef4e"
                            }
                        ]
                    }
                ]
            }
            "###,
        ) {
            Err(e) => panic!(e),
            Ok(manifest) => {
                assert_eq!(manifest.app_id, "org.gnome.SoundJuicer");
            }
        }
    }

    #[test]
    pub fn test_parse_json_with_comments() {
        match FlatpakManifest::parse(
            "manifest.json",
            r###"
            {
                "app-id": "org.gnome.SoundJuicer",
                "runtime": "org.gnome.Platform",
                "runtime-version": "master",
                "sdk": "org.gnome.Sdk",
                "command": "sound-juicer",
                "tags": [ "nightly" ],
                "desktop-file-name-suffix": " ☢️",
                "finish-args": [
                    /* X11 + XShm access */
                    "--share=ipc", "--socket=fallback-x11",
                    /* Wayland access */
                    "--socket=wayland",
                    /* audio CDs */
                    "--device=all",
                    /* Needs to talk to the network */
                    "--share=network",
                    /* Play sounds */
                    "--socket=pulseaudio",
                    /* Browse user's Music directory */
                    "--filesystem=xdg-music",
                    /* Migrate DConf settings from the host */
                    "--metadata=X-DConf=migrate-path=/org/gnome/sound-juicer/",
                    /* optical media detection */
                    "--talk-name=org.gtk.vfs", "--talk-name=org.gtk.vfs.*",
                    /* Ensure cdda gstreamer plugin is picked found for audio CD's */
                    "--env=GST_PLUGIN_PATH=/app/lib/codecs/lib/gstreamer-1.0"
                ],
                "cleanup": [ "/include", "/share/bash-completion" ],
                "modules": [
                    /* gst-plugins-base needs cdparanoia to add support for cdda */
                    {
                        "name": "cdparanoia",
                        "buildsystem": "simple",
                        "build-commands": [
                            "cp /usr/share/automake-*/config.{sub,guess} .",
                            "./configure --prefix=/app",
                            "make all slib",
                            "make install"
                        ],
                        "sources": [
                            {
                                "type": "archive",
                                "url": "http://downloads.xiph.org/releases/cdparanoia/cdparanoia-III-10.2.src.tgz",
                                "sha256": "005db45ef4ee017f5c32ec124f913a0546e77014266c6a1c50df902a55fe64df"
                            },
                            {
                                "type": "patch",
                                "path": "cdparanoia-use-proper-gnu-config-files.patch"
                            }
                        ]
                    },
                    /* To play cdda */
                    {
                        "name": "gst-plugins-base",
                        "buildsystem": "meson",
                        "config-opts": [
                            "--prefix=/app",
                            "-Dauto_features=disabled",
                            "-Dcdparanoia=enabled"
                        ],
                        "cleanup": [ "*.la", "/share/gtk-doc" ],
                        "sources": [
                            {
                                "type": "git",
                                "url": "https://gitlab.freedesktop.org/gstreamer/gst-plugins-base.git",
                                "branch" : "1.16.2",
                                "commit" : "9d3581b2e6f12f0b7e790d1ebb63b90cf5b1ef4e"
                            }
                        ]
                    }
                ]
            }
            "###,
        ) {
            Err(e) => panic!(e),
            Ok(manifest) => {
                assert_eq!(manifest.app_id, "org.gnome.SoundJuicer");
            }
        }
    }

    #[test]
    pub fn test_parse_json_with_multi_line_comments() {
        match FlatpakManifest::parse(
            "manifest.json",
            r###"
            {
              "app-id": "org.gnome.Lollypop",
              "runtime": "org.gnome.Platform",
              "runtime-version": "40",
              "sdk": "org.gnome.Sdk",
              "command": "lollypop",
              "finish-args": [
                "--share=ipc",
                "--own-name=org.mpris.MediaPlayer2.Lollypop",
                "--metadata=X-DConf=migrate-path=/org/gnome/Lollypop/"
              ],
              /* FFmpeg-full and gst-plugins-ugly required for .wma support
               * Due to possible legal stubbornness in the USA, it can't be downloaded automatically
               */
              "add-extensions": {
                "org.freedesktop.Platform.ffmpeg-full": {
                  "directory": "lib/ffmpeg",
                  "version": "20.08",
                  "add-ld-path": ".",
                  "no-autodownload": true,
                  "autodelete": false
                }
              },
              "cleanup-commands": [
                "mkdir -p /app/lib/ffmpeg"
              ],
              "modules": [
                "pypi-dependencies.json",
                {
                  "name": "gst-plugins-ugly",
                  "buildsystem": "meson",
                  "cleanup": [
                    "*.la",
                    "/share/gtk-doc"
                  ],
                  "sources": [{
                    "type": "archive",
                    "url": "https://gstreamer.freedesktop.org/src/gst-plugins-ugly/gst-plugins-ugly-1.16.2.tar.xz",
                    "sha256": "5500415b865e8b62775d4742cbb9f37146a50caecfc0e7a6fc0160d3c560fbca"
                  }]
                }
              ]
            }
            "###,
        ) {
            Err(e) => panic!(e),
            Ok(manifest) => {
                assert_eq!(manifest.app_id, "org.gnome.Lollypop");
            }
        }
    }
}
