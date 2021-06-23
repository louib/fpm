use std::collections::BTreeMap;
use std::collections::HashSet;

use fpm::flatpak_manifest::{
    FlatpakManifest, FlatpakModule, FlatpakModuleDescription, FlatpakSourceDescription,
};

fn main() {
    fpm::logger::init();
    let db = fpm::db::Database::get_database();

    let mut app_ids: HashSet<String> = HashSet::new();
    let mut sources_count: BTreeMap<String, i64> = BTreeMap::new();
    let mut sources_total_count: i64 = 0;
    let mut sources_mirror_urls_supported_count: i64 = 0;
    let mut sources_mirror_urls_available_count: i64 = 0;
    let mut sources_git_with_commit_count: i64 = 0;
    let mut sources_git_with_tag_count: i64 = 0;
    let mut sources_git_with_tag_and_commit_count: i64 = 0;
    let mut archives_urls: HashSet<String> = HashSet::new();
    let mut archives_formats: BTreeMap<String, i64> = BTreeMap::new();
    let mut project_names_from_archives: HashSet<String> = HashSet::new();
    let mut sources_archives_with_semver: i64 = 0;
    let mut sources_archives_with_direct_git_url: i64 = 0;
    let mut sources_unknown_with_project_name: i64 = 0;
    let mut sources_unknown_without_project_name: i64 = 0;
    let mut sources_archives_count: i64 = 0;
    let mut invalid_sources_count: i64 = 0;
    let mut empty_sources_count: i64 = 0;
    let mut modules_count: i64 = 0;
    let mut standalone_modules_count: i64 = 0;
    let mut modules_sources_count: BTreeMap<i32, i64> = BTreeMap::new();
    let mut modules_buildsystems_count: BTreeMap<String, i64> = BTreeMap::new();
    let mut manifests_max_depth: BTreeMap<i32, i64> = BTreeMap::new();
    let mut manifests_count: i64 = 0;
    let mut extension_manifests_count: i64 = 0;
    let mut extensions_count: BTreeMap<String, i64> = BTreeMap::new();
    let mut no_extensions_count: i64 = 0;
    let mut patched_modules_count: i64 = 0;
    let mut modules_urls_count: i64 = 0;
    let mut modules_mirror_urls_count: i64 = 0;
    let mut modules_urls_protocols: BTreeMap<String, i64> = BTreeMap::new();

    if db.indexed_projects.len() == 0 {
        panic!("There are no projects in the database!");
    }

    for (project_id, project) in &db.indexed_projects {
        // We're only interested in having stats for the projects supporting Flatpak.
        if !project.supports_flatpak() {
            continue;
        }

        log::info!("Processing project {}...", project_id);
        let repo_url = project.get_main_vcs_url();

        let repo_dir = match fpm::utils::clone_git_repo(&repo_url) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("Could not clone repo {}: {}", &repo_url, e);
                continue;
            }
        };

        for manifest_path in &project.flatpak_app_manifests {
            let absolute_manifest_path = repo_dir.to_string() + manifest_path;

            let flatpak_manifest = match FlatpakManifest::load_from_file(absolute_manifest_path.to_string()) {
                Some(m) => m,
                None => {
                    log::warn!(
                        "Could not parse Flatpak manifest at {}!!!",
                        absolute_manifest_path
                    );
                    continue;
                }
            };

            manifests_count += 1;

            app_ids.insert(flatpak_manifest.get_id());

            if flatpak_manifest.is_extension() {
                extension_manifests_count += 1;
            }

            for extension_name in &flatpak_manifest.sdk_extensions {
                let new_count = extensions_count.get(extension_name).unwrap_or(&0) + 1;
                extensions_count.insert(extension_name.to_string(), new_count);
            }
            if flatpak_manifest.sdk_extensions.len() == 0 {
                no_extensions_count += 1;
            }

            let manifest_depth = flatpak_manifest.get_max_depth();
            let new_count = manifests_max_depth.get(&manifest_depth).unwrap_or(&0) + 1;
            manifests_max_depth.insert(manifest_depth, new_count);

            // FIXME we should get the modules recursively here!!!
            for module in &flatpak_manifest.modules {
                let module_description = match &module {
                    FlatpakModule::Path(_) => continue,
                    FlatpakModule::Description(d) => d,
                };

                // We're only counting inlined module descriptions for now.
                modules_count += 1;
                for url in module.get_all_repos_urls() {
                    let url = url.trim();
                    modules_urls_count += 1;
                    if url == "." || url == ".." || url.starts_with("./") || url.starts_with("../") {
                        let new_modules_protocol_count =
                            modules_urls_protocols.get("relative fs path").unwrap_or(&0) + 1;
                        modules_urls_protocols
                            .insert("relative fs path".to_string(), new_modules_protocol_count);
                    } else if url.starts_with("http://") {
                        let new_modules_protocol_count = modules_urls_protocols.get("http").unwrap_or(&0) + 1;
                        modules_urls_protocols.insert("http".to_string(), new_modules_protocol_count);
                    } else if url.starts_with("https://") {
                        let new_modules_protocol_count =
                            modules_urls_protocols.get("https").unwrap_or(&0) + 1;
                        modules_urls_protocols.insert("https".to_string(), new_modules_protocol_count);
                    } else if url.starts_with("git://") {
                        let new_modules_protocol_count = modules_urls_protocols.get("git").unwrap_or(&0) + 1;
                        modules_urls_protocols.insert("git".to_string(), new_modules_protocol_count);
                    } else if url.starts_with("ftp://") {
                        let new_modules_protocol_count = modules_urls_protocols.get("ftp").unwrap_or(&0) + 1;
                        modules_urls_protocols.insert("ftp".to_string(), new_modules_protocol_count);
                    } else if url.starts_with("svn://") {
                        let new_modules_protocol_count = modules_urls_protocols.get("svn").unwrap_or(&0) + 1;
                        modules_urls_protocols.insert("svn".to_string(), new_modules_protocol_count);
                    } else if url.starts_with("file://") {
                        let new_modules_protocol_count = modules_urls_protocols.get("file").unwrap_or(&0) + 1;
                        modules_urls_protocols.insert("file".to_string(), new_modules_protocol_count);
                    } else {
                        let new_modules_protocol_count =
                            modules_urls_protocols.get("other").unwrap_or(&0) + 1;
                        modules_urls_protocols.insert("other".to_string(), new_modules_protocol_count);
                        log::warn!("UNKNOWN URL PROTOCOL {}", url);
                    }
                }

                for url in module.get_repos_mirror_urls() {
                    modules_mirror_urls_count += 1;
                }

                if module.is_patched() {
                    patched_modules_count += 1;
                }

                let module_sources_count = module.get_sources_count() as i32;
                let new_sources_count = modules_sources_count.get(&module_sources_count).unwrap_or(&0) + 1;
                modules_sources_count.insert(module_sources_count, new_sources_count);

                if let Some(buildsystem) = module_description.get_buildsystem() {
                    let new_buildsystem_count =
                        modules_buildsystems_count.get(&buildsystem).unwrap_or(&0) + 1;
                    modules_buildsystems_count.insert(buildsystem.to_string(), new_buildsystem_count);
                } else {
                    let new_buildsystem_count =
                        modules_buildsystems_count.get("unspecified").unwrap_or(&0) + 1;
                    modules_buildsystems_count.insert("unspecified".to_string(), new_buildsystem_count);
                }

                for source in &module_description.sources {
                    sources_total_count += 1;

                    if source.supports_mirror_urls() {
                        sources_mirror_urls_supported_count += 1;
                    }
                    if source.get_all_mirror_urls().len() != 0 {
                        sources_mirror_urls_available_count += 1;
                    }

                    let source_type_name = source.get_type_name();
                    let new_count = sources_count.get(&source_type_name).unwrap_or(&0) + 1;
                    sources_count.insert(source_type_name.to_string(), new_count);

                    if source_type_name == "git" && source.has_commit() {
                        sources_git_with_commit_count += 1;
                    }
                    if source_type_name == "git" && source.has_tag() {
                        sources_git_with_tag_count += 1;
                    }
                    if source_type_name == "git" && source.has_tag() && source.has_commit() {
                        sources_git_with_tag_and_commit_count += 1;
                    }

                    if !source.type_is_valid() {
                        invalid_sources_count += 1;
                    }

                    if source.type_is_empty() {
                        empty_sources_count += 1;
                    }

                    for url in source.get_all_urls() {
                        if source_type_name == "git" {
                            log::debug!("GIT URL {}", url);
                        } else if source_type_name == "archive" {
                            if archives_urls.contains(&url) {
                                continue;
                            }
                            archives_urls.insert(url.to_string());

                            log::debug!("ARCHIVE URL {}", url);
                            sources_archives_count += 1;

                            if let Some(archive_type) = FlatpakSourceDescription::detect_archive_type(&url) {
                                let new_count = archives_formats.get(&archive_type).unwrap_or(&0) + 1;
                                archives_formats.insert(archive_type.to_string(), new_count);
                            } else {
                                let new_count = archives_formats.get("unknown").unwrap_or(&0) + 1;
                                archives_formats.insert("unknown".to_string(), new_count);
                            }

                            if fpm::utils::get_semver_from_archive_url(&url).is_some() {
                                sources_archives_with_semver += 1;
                            } else {
                                continue;
                            }
                            if fpm::utils::get_git_url_from_archive_url(&url).is_some() {
                                sources_archives_with_direct_git_url += 1;
                            } else {
                                log::debug!("ARCHIVE URL FROM UNKNOWN SOURCE {}", url);
                                if let Some(project_name) =
                                    fpm::utils::get_project_name_from_archive_url(&url)
                                {
                                    sources_unknown_with_project_name += 1;
                                    project_names_from_archives.insert(project_name.to_lowercase());
                                } else {
                                    sources_unknown_without_project_name += 1;
                                }
                            }
                        }
                    }
                }
            }
        }

        for manifest_path in &project.flatpak_module_manifests {
            let absolute_manifest_path = repo_dir.to_string() + manifest_path;
            let module_description =
                FlatpakModuleDescription::load_from_file(absolute_manifest_path).unwrap();
            standalone_modules_count += 1;
        }
    }
    println!("Unique app IDs: {}", app_ids.len());

    println!("===== Manifests =====");
    println!("Total count: {}", manifests_count);
    for (depth, depth_count) in manifests_max_depth {
        println!(
            "Depth {}: {:.2}% ({}/{})",
            depth,
            (depth_count as f64 / manifests_count as f64) * 100.0,
            depth_count,
            manifests_count,
        );
    }
    println!("Number of extension manifests: {}.", extension_manifests_count);
    for (extension_name, count) in extensions_count {
        println!(
            "Extension {}: {:.2}% ({}/{})",
            extension_name,
            (count as f64 / manifests_count as f64) * 100.0,
            count,
            manifests_count,
        );
    }
    println!(
        "Manifests with no SDK extensions: {:.2}% ({}/{})",
        (no_extensions_count as f64 / manifests_count as f64) * 100.0,
        no_extensions_count,
        manifests_count,
    );
    println!("=====================");
    println!("\n");

    println!("===== Modules =====");
    println!("Total count: {}", modules_count);
    println!("Standalone modules: {}", standalone_modules_count);
    println!(
        "Patched modules: {:.2}% ({}/{})",
        (patched_modules_count as f64 / modules_count as f64) * 100.0,
        patched_modules_count,
        modules_count,
    );
    for (source_count, count) in modules_sources_count {
        println!(
            "Modules with {} source(s): {:.2}% ({}/{})",
            source_count,
            (count as f64 / modules_count as f64) * 100.0,
            count,
            sources_total_count,
        );
    }
    for (buildsystem, buildsystem_count) in modules_buildsystems_count {
        println!(
            "Modules with buildsystem {}: {:.2}% ({}/{})",
            buildsystem,
            (buildsystem_count as f64 / modules_count as f64) * 100.0,
            buildsystem_count,
            modules_count
        );
    }
    println!("=====================");
    println!("\n");

    println!("===== Sources =====");
    println!("Total count: {}", sources_total_count);
    for (source_type, source_count) in &sources_count {
        println!(
            "{}: {:.2}% ({}/{})",
            source_type,
            (*source_count as f64 / sources_total_count as f64) * 100.0,
            source_count,
            sources_total_count,
        );
    }
    println!(
        "Sources with mirror urls: {:.2}% ({}/{})",
        (sources_mirror_urls_available_count as f64 / sources_mirror_urls_supported_count as f64) * 100.0,
        sources_mirror_urls_available_count,
        sources_mirror_urls_supported_count,
    );
    let sources_git_count = sources_count.get("git").unwrap();
    println!(
        "Git sources fixed with commit hash: {:.2}% ({}/{})",
        (sources_git_with_commit_count as f64 / *sources_git_count as f64) * 100.0,
        sources_git_with_commit_count,
        sources_git_count
    );
    println!(
        "Git sources fixed with tag: {:.2}% ({}/{})",
        (sources_git_with_tag_count as f64 / *sources_git_count as f64) * 100.0,
        sources_git_with_tag_count,
        sources_git_count
    );
    println!(
        "Git sources fixed with tag and commit: {:.2}% ({}/{})",
        (sources_git_with_tag_and_commit_count as f64 / *sources_git_count as f64) * 100.0,
        sources_git_with_tag_and_commit_count,
        sources_git_count
    );
    println!(
        "Archive URLS with a semver: {:.2}% ({}/{})",
        (sources_archives_with_semver as f64 / sources_archives_count as f64) * 100.0,
        sources_archives_with_semver,
        sources_archives_count,
    );
    println!(
        "Versioned archive URLS with a direct git repository: {:.2}% ({}/{})",
        (sources_archives_with_direct_git_url as f64 / sources_archives_with_semver as f64) * 100.0,
        sources_archives_with_direct_git_url,
        sources_archives_with_semver,
    );
    println!(
        "Versioned archive URLS without a direct git repository but with a project name: {:.2}% ({}/{})",
        (sources_unknown_with_project_name as f64 / sources_archives_with_semver as f64) * 100.0,
        sources_unknown_with_project_name,
        sources_archives_with_semver,
    );
    println!(
        "Versioned archive URLS without a direct git repository but without a project name: {:.2}% ({}/{})",
        (sources_unknown_without_project_name as f64 / sources_archives_with_semver as f64) * 100.0,
        sources_unknown_without_project_name,
        sources_archives_with_semver,
    );
    println!(
        "Unique project names extracted from archives: {}.",
        project_names_from_archives.len(),
    );
    for (archive_format, archive_count) in &archives_formats {
        println!(
            "Archives with {} format: {:.2}% ({}/{})",
            archive_format,
            (*archive_count as f64 / sources_archives_count as f64) * 100.0,
            archive_count,
            sources_archives_count,
        );
    }
    println!("Sources with invalid type: {}.", invalid_sources_count);
    println!("Sources with empty type: {}.", empty_sources_count);
    println!("=====================");
    println!("\n");

    println!("===== URLs =====");
    println!("Total count: {}", modules_urls_count);
    for (protocol_name, count) in modules_urls_protocols {
        println!(
            "URLs with protocol {}: {:.2}% ({}/{})",
            protocol_name,
            (count as f64 / modules_urls_count as f64) * 100.0,
            count,
            modules_urls_count
        );
    }
    println!(
        "URLs used as mirrors: {:.2}% ({}/{})",
        (modules_mirror_urls_count as f64 / modules_urls_count as f64) * 100.0,
        modules_mirror_urls_count,
        modules_urls_count,
    );
    println!("=====================");
}
