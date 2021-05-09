use std::env;

use futures::executor::block_on;
use reqwest::header;
use serde::{Deserialize, Serialize};

// See https://docs.github.com/en/rest/reference/repos
#[derive(Debug, Serialize, Deserialize)]
pub struct GitHubRepo {
    pub id: String,
    pub name: String,
    pub full_name: String,
    pub description: String,
    pub fork: bool,
    pub is_template: Option<bool>,
    pub archived: Option<bool>,
    pub disabled: Option<bool>,
    pub topics: Option<Vec<String>>,
    pub clone_url: Option<String>,
    pub git_url: Option<String>,
    pub homepage: Option<String>,
    pub forks_count: Option<i64>,
    pub stargazers_count: Option<i64>,
    pub watchers_count: Option<i64>,
    pub size: Option<i64>,
    pub default_branch: Option<String>,
}
impl GitHubRepo {
    pub fn to_software_project(self) -> fpm::projects::SoftwareProject {
        let mut project = fpm::projects::SoftwareProject::default();
        let git_url = format!("https://github.com/{}.git", self.full_name);
        project.id = fpm::utils::repo_url_to_reverse_dns(&git_url);
        project.name = self.name;
        project.default_branch = self.default_branch;
        project.description = self.description;
        project.vcs_urls.push(git_url);
        if let Some(topics) = self.topics {
            project.keywords = topics;
        }
        project
    }
    pub fn get_git_url(&self) -> String {
        format!("https://github.com/{}.git", self.full_name)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GitHubError {
    pub message: String,
    pub documentation_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GitHubRepoSearchResponse {
    pub items: Vec<GitHubRepo>,
    pub total_count: i64,
    pub incomplete_results: bool,
}

pub fn search_repos(search_term: &str) -> Vec<GitHubRepo> {
    let mut projects: Vec<GitHubRepo> = vec![];

    // Using a search query with the repository search feature of GitHub
    // will by default search in the title, description and README.
    let next_page_url = format!(
        "https://api.github.com/search/repositories?type=all&per_page=100&q={}+in:readme",
        search_term,
    );

    let client = get_github_client();

    log::info!("Search GitHub for term {}.", search_term);
    while !next_page_url.is_empty() {
        // TODO make this really asynchronous with async/await.
        let response = match client.get(&next_page_url).send() {
            Ok(r) => r,
            Err(e) => {
                log::error!("Could not fetch GitHub url {}: {}.", next_page_url, e);
                return projects;
            }
        };

        if response.status().as_u16() == 204 {
            return projects;
        }

        if response.status().as_u16() > 399 {
            let error_object: GitHubError = match serde_yaml::from_str(&response.text().unwrap()) {
                Ok(e) => e,
                Err(e) => {
                    log::error!("Could not parse GitHub error {}.", e);
                    return projects;
                }
            };
            log::error!("Error returned by the GitHub API: {}", error_object.message);
            return projects;
        }

        let link_header = match &response.headers().get("link") {
            Some(h) => h.to_str().unwrap(),
            None => "",
        };
        let next_page_url = fpm::utils::get_next_page_url(link_header);

        let response_content = response.text().unwrap();
        let response: GitHubRepoSearchResponse = match serde_yaml::from_str(&response_content) {
            Ok(p) => p,
            Err(e) => {
                log::error!("Could not parse GitHub repo search response {}: {}.", e, &response_content);
                return projects;
            }
        };
        for github_project in response.items {
            if github_project.fork {
                continue;
            }
            log::debug!("Adding GitHub repo {}.", github_project.name);
            projects.push(github_project);
        }
    }
    projects
}

pub fn get_org_repos(org_name: &str) -> Vec<fpm::projects::SoftwareProject> {
    let mut paged_response = get_repos(fpm::utils::PagedRequest {
        domain: "".to_string(),
        token: None,
        next_page_url: Some(format!(
            "https://api.github.com/orgs/{}/repos?type=all&per_page=100",
            org_name
        )),
    });
    let mut all_projects = vec![];
    let mut projects = paged_response.results;
    while projects.len() > 0 {
        for project in projects {
            log::debug!("Adding project {}.", &project.name);
            all_projects.push(project);
        }

        if paged_response.next_page_url.is_none() {
            break;
        }

        paged_response = get_repos(fpm::utils::PagedRequest {
            domain: "".to_string(),
            token: None,
            next_page_url: paged_response.next_page_url,
        });
        projects = paged_response.results;
    }
    all_projects
}

pub fn get_and_add_repos(db: &mut fpm::db::Database) {
    log::info!("Getting all projects from github.com");
    let mut request = fpm::utils::PagedRequest {
        domain: "".to_string(),
        token: None,
        next_page_url: None,
    };
    let mut paged_response = get_repos(request);

    let mut projects = paged_response.results;
    while projects.len() > 0 {
        for project in projects {
            log::info!("Adding project {}.", &project.name);
            db.add_project(project);
        }

        if paged_response.next_page_url.is_none() {
            break;
        }

        paged_response = get_repos(fpm::utils::PagedRequest {
            domain: "".to_string(),
            token: paged_response.token,
            next_page_url: paged_response.next_page_url,
        });
        projects = paged_response.results;
    }
}

pub fn get_repos(
    request: fpm::utils::PagedRequest,
) -> fpm::utils::PagedResponse<fpm::projects::SoftwareProject> {
    // By default, we get all the repos.
    let mut current_url = format!("https://api.github.com/repositories?type=all&per_page=2");
    if let Some(url) = request.next_page_url {
        current_url = url;
    }

    let mut projects: Vec<fpm::projects::SoftwareProject> = vec![];
    let default_response = fpm::utils::PagedResponse::<fpm::projects::SoftwareProject> {
        results: vec![],
        token: None,
        next_page_url: None,
    };

    let client = get_github_client();

    log::info!("Getting GitHub projects page at {}.", current_url);
    // TODO make this really asynchronous with async/await.
    let response = match client.get(&current_url).send() {
        Ok(r) => r,
        Err(e) => {
            log::error!("Could not fetch GitHub url {}: {}.", current_url, e);
            return default_response;
        }
    };

    if response.status().as_u16() == 204 {
        return default_response;
    }

    if response.status().as_u16() > 399 {
        let error_object: GitHubError = match serde_yaml::from_str(&response.text().unwrap()) {
            Ok(e) => e,
            Err(e) => {
                log::error!("Could not parse GitHub error {}.", e);
                return default_response;
            }
        };
        log::error!("Error returned by the GitHub API: {}", error_object.message);
        return default_response;
    }

    let link_header = match &response.headers().get("link") {
        Some(h) => h.to_str().unwrap(),
        None => "",
    };
    let next_page_url = fpm::utils::get_next_page_url(link_header);

    let response_content = response.text().unwrap();
    let github_repos: Vec<GitHubRepo> = match serde_yaml::from_str(&response_content) {
        Ok(p) => p,
        Err(e) => {
            log::error!("Could not parse GitHub repos {}.", e);
            return default_response;
        }
    };
    for github_project in github_repos {
        if github_project.fork {
            continue;
        }
        log::debug!("Adding GitHub repo {}.", github_project.name);
        projects.push(github_project.to_software_project());
    }

    fpm::utils::PagedResponse::<fpm::projects::SoftwareProject> {
        results: projects,
        token: None,
        next_page_url: next_page_url,
    }
}

pub fn get_github_client() -> reqwest::blocking::Client {
    let mut headers = header::HeaderMap::new();
    // User agent is required when using the GitHub API.
    // See https://docs.github.com/en/rest/overview/resources-in-the-rest-api#user-agent-required
    headers.insert("User-Agent", header::HeaderValue::from_str("fpm").unwrap());
    headers.insert(
        "Accept",
        header::HeaderValue::from_str("application/vnd.github.v3+json").unwrap(),
    );
    if let Ok(token) = env::var("FPM_GITHUB_TOKEN") {
        let auth_header_value = format!("token {}", &token);
        headers.insert(
            "Authorization",
            header::HeaderValue::from_str(&auth_header_value.to_string()).unwrap(),
        );
    } else {
        log::warn!("No GitHub API token located at FPM_GITHUB_TOKEN. We will get rate limited faster.");
    }

    reqwest::blocking::Client::builder()
        .default_headers(headers)
        .build()
        .unwrap()
}
