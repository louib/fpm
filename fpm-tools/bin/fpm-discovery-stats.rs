use std::collections::BTreeMap;
use std::collections::HashSet;

use fpm::flatpak_manifest::{
    FlatpakManifest, FlatpakModule, FlatpakModuleDescription, FlatpakSource, FlatpakSourceDescription,
};

fn main() {
    fpm::logger::init();
    let db = fpm::db::Database::get_database();

    let mut app_ids_to_sources: BTreeMap<String, HashSet<String>> = BTreeMap::new();
    let mut sources_repos_count: BTreeMap<String, i64> = BTreeMap::new();
    let mut sources_repos_with_manifests_count: BTreeMap<String, i64> = BTreeMap::new();
    let mut sources_manifests_count: BTreeMap<String, i64> = BTreeMap::new();
    let mut sources_repos_with_modules_count: BTreeMap<String, i64> = BTreeMap::new();
    let mut sources_modules_count: BTreeMap<String, i64> = BTreeMap::new();

    if db.indexed_projects.len() == 0 {
        panic!("There are no projects in the database!");
    }

    for (project_id, project) in &db.indexed_projects {
        log::info!("Processing project {}...", project_id);
        for source in &project.sources {
            let new_repos_count = sources_repos_count.get(source).unwrap_or(&0) + 1;
            sources_repos_count.insert(source.to_string(), new_repos_count);

            if project.flatpak_app_manifests.len() > 0 {
                let new_repos_with_manifest_count =
                    sources_repos_with_manifests_count.get(source).unwrap_or(&0) + 1;
                sources_repos_with_manifests_count.insert(source.to_string(), new_repos_with_manifest_count);

                let new_manifest_count = sources_manifests_count.get(source).unwrap_or(&0) + 1;
                sources_manifests_count.insert(source.to_string(), new_manifest_count);
            }

            if project.flatpak_module_manifests.len() > 0 {
                let new_repos_with_module_count =
                    sources_repos_with_modules_count.get(source).unwrap_or(&0) + 1;
                sources_repos_with_modules_count.insert(source.to_string(), new_repos_with_module_count);

                let new_module_count = sources_modules_count.get(source).unwrap_or(&0) + 1;
                sources_modules_count.insert(source.to_string(), new_module_count);
            }

            if !app_ids_to_sources.get(source).is_some() {
                app_ids_to_sources.insert(source.to_string(), HashSet::new());
            }
            app_ids_to_sources
                .get_mut(source)
                .unwrap()
                .insert(source.to_string());
        }
    }

    for (source_name, source_repos_count) in sources_repos_count {
        let repos_with_manifests_count = sources_repos_with_manifests_count.get(&source_name).unwrap_or(&0);
        let repos_with_modules_count = sources_repos_with_modules_count.get(&source_name).unwrap_or(&0);
        let manifests_count = sources_manifests_count.get(&source_name).unwrap_or(&0);
        let modules_count = sources_modules_count.get(&source_name).unwrap_or(&0);

        println!("===== {} =====", source_name);
        println!(
            "Repositories with Flatpak app manifests: {:.2}% ({}/{})",
            (*repos_with_manifests_count as f64 / source_repos_count as f64) * 100.0,
            repos_with_manifests_count,
            source_repos_count,
        );
        println!(
            "Repositories with Flatpak module manifests: {:.2}% ({}/{})",
            (*repos_with_modules_count as f64 / source_repos_count as f64) * 100.0,
            repos_with_modules_count,
            source_repos_count,
        );
        println!("=====================");
        println!("\n");
    }
}
