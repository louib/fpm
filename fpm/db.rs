use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path;

use crate::flatpak_manifest::FlatpakModuleDescription;
use crate::projects::SoftwareProject;

pub const DEFAULT_DB_PATH: &str = ".fpm-db";
pub const MODULES_DB_SUBDIR: &str = "/modules";
pub const PROJECTS_DB_SUBDIR: &str = "/projects";
pub const REPOS_DB_SUBDIR: &str = "/repositories";

pub struct Database {
    pub modules: Vec<FlatpakModuleDescription>,
    pub indexed_projects: BTreeMap<String, SoftwareProject>,
}
impl Database {
    pub fn get_database() -> Database {
        if let Err(e) = fs::create_dir_all(Database::get_modules_db_path()) {
            panic!("Could not initialize database directory: {}.", e);
        }
        if let Err(e) = fs::create_dir_all(Database::get_projects_db_path()) {
            panic!("Could not initialize database directory: {}.", e);
        }
        if let Err(e) = fs::create_dir_all(Database::get_repos_db_path()) {
            panic!("Could not initialize database directory: {}.", e);
        }
        let mut indexed_projects: BTreeMap<String, SoftwareProject> = BTreeMap::new();
        for project in Database::get_all_projects() {
            indexed_projects.insert(project.id.clone(), project);
        }
        // FIXME error handle the init.
        Database {
            modules: Database::get_all_modules(),
            indexed_projects: indexed_projects,
        }
    }

    pub fn get_db_path() -> String {
        if let Ok(path) = env::var("FPM_DB_PATH") {
            return path.to_string();
        }
        if let Ok(path) = env::var("HOME") {
            return path + "/" + &DEFAULT_DB_PATH.to_string();
        }
        return DEFAULT_DB_PATH.to_string();
    }

    pub fn get_modules_db_path() -> String {
        Database::get_db_path() + MODULES_DB_SUBDIR
    }

    pub fn get_projects_db_path() -> String {
        Database::get_db_path() + PROJECTS_DB_SUBDIR
    }

    pub fn get_repos_db_path() -> String {
        Database::get_db_path() + REPOS_DB_SUBDIR
    }

    pub fn get_all_projects() -> Vec<SoftwareProject> {
        let projects_path = Database::get_projects_db_path();
        let projects_path = path::Path::new(&projects_path);
        let all_projects_paths = match crate::utils::get_all_paths(projects_path) {
            Ok(paths) => paths,
            Err(e) => {
                log::error!("Could not get projects' paths: {}", e);
                return vec![];
            }
        };
        let mut projects: Vec<SoftwareProject> = vec![];
        for project_path in all_projects_paths.iter() {
            let project_path_str = project_path.to_str().unwrap();
            if !project_path.is_file() {
                log::debug!("{} is not a file.", &project_path_str);
                continue;
            }
            // Don't even try to open it if it's not a yaml file.
            if !project_path_str.ends_with("yml") && !project_path_str.ends_with("yaml") {
                continue;
            }
            let project_content = match fs::read_to_string(project_path) {
                Ok(content) => content,
                Err(e) => {
                    log::debug!("Could not read project file {}: {}.", &project_path_str, e);
                    continue;
                }
            };
            let project = match serde_yaml::from_str(&project_content) {
                Ok(p) => p,
                Err(e) => {
                    log::debug!("Could not parse project file at {}: {}.", &project_path_str, e);
                    continue;
                }
            };
            projects.push(project);
        }
        projects
    }

    pub fn get_all_modules() -> Vec<FlatpakModuleDescription> {
        let modules_path = Database::get_modules_db_path();
        let modules_path = path::Path::new(&modules_path);
        let all_modules_paths = match crate::utils::get_all_paths(modules_path) {
            Ok(paths) => paths,
            Err(e) => {
                log::error!("Could not get modules from database: {}.", e);
                return vec![];
            }
        };
        let mut modules: Vec<FlatpakModuleDescription> = vec![];
        for module_path in all_modules_paths.iter() {
            let module_path_str = module_path.to_str().unwrap();
            if !module_path.is_file() {
                log::debug!("{} is not a file.", &module_path_str);
                continue;
            }
            // Don't even try to open it if it's not a yaml file.
            if !module_path_str.ends_with("yml") && !module_path_str.ends_with("yaml") {
                continue;
            }
            let module_content = match fs::read_to_string(module_path) {
                Ok(content) => content,
                Err(e) => {
                    log::debug!("Could not read module file {}: {}.", &module_path_str, e);
                    continue;
                }
            };
            let module = match serde_yaml::from_str(&module_content) {
                Ok(m) => m,
                Err(e) => {
                    log::debug!("Could not parse module file at {}: {}.", &module_path_str, e);
                    continue;
                }
            };
            modules.push(module);
        }
        modules
    }

    pub fn search_modules(&self, search_term: &str) -> Vec<&FlatpakModuleDescription> {
        let mut modules: Vec<&FlatpakModuleDescription> = vec![];
        for module in &self.modules {
            if module.name.to_lowercase().contains(&search_term.to_lowercase()) {
                modules.push(&module);
            }
        }
        modules
    }

    pub fn remove_module() {}

    pub fn add_module(&mut self, new_module: FlatpakModuleDescription) {
        let module_hash = new_module.get_hash();
        let modules_path = Database::get_modules_db_path();
        let new_module_path = format!("{}/{}.yaml", modules_path, module_hash,);
        log::info!("Adding module at {}", new_module_path);
        let new_module_fs_path = path::Path::new(&new_module_path);
        if new_module_fs_path.exists() {
            // The path is based on a hash of the module, so there should be no need to
            // update a file that exists.
            return;
        }
        match fs::write(new_module_fs_path, serde_yaml::to_string(&new_module).unwrap()) {
            Ok(content) => content,
            Err(e) => {
                eprintln!("Could not write new module at {}: {}", new_module_path.to_string(), e);
            }
        };
        self.modules.push(new_module);
    }

    pub fn update_project(&mut self, project: &SoftwareProject) {
        let projects_path = Database::get_projects_db_path();
        if project.id.len() == 0 {
            panic!("Trying to update a project to the db without an id!");
        }
        let existing_project = self.indexed_projects.get_mut(&project.id).unwrap();

        let new_project_path = format!("{}/{}.yaml", projects_path, &project.id);
        let project_fs_path = path::Path::new(&new_project_path);
        if !project_fs_path.exists() {
            panic!("Project {} does not exist", project.id);
        }
        log::info!("Updating project at {}", new_project_path);

        existing_project.merge(project);
        match fs::write(project_fs_path, serde_yaml::to_string(&existing_project).unwrap()) {
            Ok(content) => content,
            Err(e) => {
                eprintln!("Could not write new project at {}: {}", new_project_path.to_string(), e);
            }
        };
    }

    pub fn add_project(&mut self, project: SoftwareProject) {
        let projects_path = Database::get_projects_db_path();
        if project.id.len() == 0 {
            panic!("Trying to add a project to the db without an id!");
        }
        let project_path = format!("{}/{}.yaml", projects_path, &project.id);
        log::info!("Adding project at {}", project_path);
        let new_project_fs_path = path::Path::new(&project_path);
        if new_project_fs_path.exists() {
            return self.update_project(&project);
        }
        match fs::write(new_project_fs_path, serde_yaml::to_string(&project).unwrap()) {
            Ok(content) => content,
            Err(e) => {
                eprintln!("Could not write new project at {}: {}", project_path.to_string(), e);
            }
        };
        self.indexed_projects.insert(project.id.clone(), project);
    }

    pub fn search_projects(&self, search_term: &str) -> Vec<&SoftwareProject> {
        let mut projects: Vec<&SoftwareProject> = vec![];
        for (_, project) in &self.indexed_projects {
            if project.name.contains(&search_term) {
                projects.push(&project);
            }
        }
        projects
    }

    pub fn get_project(&self, project_id: &str) -> Option<&SoftwareProject> {
        self.indexed_projects.get(project_id)
    }

    pub fn has_project(&self, project_id: &str) -> bool {
        self.indexed_projects.contains_key(project_id)
    }
}
