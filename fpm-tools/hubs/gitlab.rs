use std::env;

use reqwest::header;

use serde::{Deserialize, Serialize};

/// The url should have the format
/// https://oauth2:TOKEN@gitlab.com/username/myrepo.git
///```
///let gitlab_repo_url = "https://gitlab.com/username/myrepo.git";
///assert_eq!(
///  fpm_tools::hubs::gitlab::add_auth_token_to_repo_url(gitlab_repo_url, "MON_PAT"),
///  "https://oauth2:MON_PAT@gitlab.com/username/myrepo.git"
///);
///```
pub fn add_auth_token_to_repo_url(repo_url: &str, auth_token: &str) -> String {
    return repo_url.replace("https://", &format!("https://oauth2:{}@", auth_token));
}

// GitLab API described here
// https://docs.gitlab.com/ee/api/projects.html
#[derive(Debug, Serialize, Deserialize)]
pub struct GitLabProject {
    pub id: String,
    pub name: String,
    pub name_with_namespace: String,
    pub created_at: String,
    pub last_activity_at: String,
    pub forks_count: i32,
    pub star_count: i32,
    pub description: Option<String>,
    pub default_branch: Option<String>,
    pub ssh_url_to_repo: String,
    pub http_url_to_repo: String,
    pub readme_url: String,
    pub tag_list: Vec<String>,
    // From the API doc:
    // If the project is a fork, and you provide a valid token to authenticate,
    // the forked_from_project field appears in the response.
    pub forked_from_project: Option<GitLabParentProject>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GitLabParentProject {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GitLabError {
    pub message: Option<String>,
    pub error: Option<String>,
}
impl GitLabError {
    pub fn get_error_message(&self) -> String {
        if let Some(m) = &self.message {
            return m.to_string();
        }
        if let Some(e) = &self.error {
            return e.to_string();
        }
        "unknown error".to_string()
    }
}

pub fn search_repos(search_term: &str) -> Vec<GitLabProject> {
    let mut projects: Vec<GitLabProject> = vec![];

    // https://docs.gitlab.com/ee/api/search.html
    let mut next_page_url = format!(
        "https://gitlab.com/api/v4/search?scope=projects&search={}",
        search_term,
    );

    let mut headers = header::HeaderMap::new();
    if let Ok(token) = env::var("FPM_GITLAB_TOKEN") {
        let auth_header_value = format!("Bearer {}", &token);
        headers.insert(
            "Authorization",
            header::HeaderValue::from_str(&auth_header_value.to_string()).unwrap(),
        );
    } else {
        log::warn!("No GitLab API token located at FPM_GITLAB_TOKEN. Aborting.");
        return projects;
    }
    let client = reqwest::blocking::Client::builder()
        .default_headers(headers)
        .build()
        .unwrap();

    log::info!("Search GitLab for term {}.", search_term);
    while !next_page_url.is_empty() {
        log::info!("Calling GitLab API at {}", &next_page_url);
        // TODO make this really asynchronous with async/await.
        let response = match client.get(&next_page_url).send() {
            Ok(r) => r,
            Err(e) => {
                log::error!("Could not fetch GitLab url {}: {}.", next_page_url, e);
                return projects;
            }
        };

        if response.status().as_u16() == 204 {
            return projects;
        }

        if response.status().as_u16() > 399 {
            let response_content = response.text().unwrap();
            let error_object: GitLabError = match serde_yaml::from_str(&response_content) {
                Ok(e) => e,
                Err(e) => {
                    log::error!("Could not parse GitLab error {}. {}", e, &response_content);
                    return projects;
                }
            };
            log::error!(
                "Error returned by the GitLab API: {}",
                error_object.get_error_message()
            );
            return projects;
        }

        let link_header = match &response.headers().get("link") {
            Some(h) => h.to_str().unwrap(),
            None => "",
        };
        next_page_url = match fpm::utils::get_next_page_url(link_header) {
            Some(u) => u,
            None => "".to_string(),
        };

        let response_content = response.text().unwrap();
        let response: Vec<GitLabProject> = match serde_yaml::from_str(&response_content) {
            Ok(p) => p,
            Err(e) => {
                log::error!(
                    "Could not parse GitLab project search response {}: {}.",
                    e,
                    &response_content
                );
                return projects;
            }
        };

        for gitlab_project in response {
            if gitlab_project.forked_from_project.is_some() {
                continue;
            }
            log::debug!("Adding GitLab repo {}.", gitlab_project.name);
            projects.push(gitlab_project);
        }
    }
    projects
}

pub fn get_all_repos(domain: &str, token_env_var_name: &str) -> Vec<GitLabProject> {
    log::info!("Getting all projects from GitLab instance at {}.", domain);
    let mut repos: Vec<GitLabProject> = vec![];
    let mut request = fpm::utils::PagedRequest {
        domain: domain.to_string(),
        token: None,
        next_page_url: None,
    };
    if let Ok(token) = env::var(token_env_var_name) {
        // See https://docs.gitlab.com/ee/api/#oauth2-tokens
        // for documentation on OAuth authentication.
        request.token = Some(token);
    } else {
        log::warn!(
            "No GitLab API token located at {} for instance at {}. Aborting.",
            token_env_var_name,
            domain
        );
        return repos;
    }
    let mut paged_response = get_repos(request);

    let mut projects = paged_response.results;
    while projects.len() > 0 {
        for project in projects {
            log::debug!("Adding project {}.", &project.name);
            repos.push(project);
        }

        if paged_response.next_page_url.is_none() {
            break;
        }

        paged_response = get_repos(fpm::utils::PagedRequest {
            domain: domain.to_string(),
            token: paged_response.token,
            next_page_url: paged_response.next_page_url,
        });
        projects = paged_response.results;
    }
    repos
}

pub fn get_repos(request: fpm::utils::PagedRequest) -> fpm::utils::PagedResponse<GitLabProject> {
    let mut current_url = format!(
        "https://{}/api/v4/projects?per_page=100&simple=false",
        request.domain
    );
    if let Some(url) = request.next_page_url {
        current_url = url;
    }

    let mut projects: Vec<GitLabProject> = vec![];
    let default_response = fpm::utils::PagedResponse::<GitLabProject> {
        results: vec![],
        token: None,
        next_page_url: None,
    };

    let mut headers = header::HeaderMap::new();
    let auth_header_value = format!("Bearer {}", request.token.as_ref().unwrap());
    let auth_header = header::HeaderValue::from_str(&auth_header_value.to_string()).unwrap();
    headers.insert("Authorization", auth_header);
    let client = reqwest::blocking::Client::builder()
        .default_headers(headers)
        .build()
        .unwrap();

    log::info!("Getting GitLab projects page at {}.", current_url);
    // TODO make this really asynchronous with async/await.
    let mut response = match client.get(&current_url).send() {
        Ok(r) => r,
        Err(e) => {
            log::error!("Could not fetch GitLab url {}: {}.", current_url, e);
            return default_response;
        }
    };

    if response.status().as_u16() == 204 {
        return default_response;
    }

    let response_headers = response.headers();

    let link_header = match &response_headers.get("link") {
        Some(h) => h.to_str().unwrap(),
        None => "",
    };
    let next_page_url = fpm::utils::get_next_page_url(link_header);

    let gitlab_projects: Vec<GitLabProject> = match serde_yaml::from_str(&response.text().unwrap()) {
        Ok(p) => p,
        Err(e) => {
            log::error!("Could not parse gitlab projects {}.", e);
            return default_response;
        }
    };
    for gitlab_project in gitlab_projects {
        if let Some(parent_project) = gitlab_project.forked_from_project {
            log::debug!("Skipping forked project {}.", &gitlab_project.name);
            continue;
        }
        log::debug!("Adding GitLab project {}.", gitlab_project.name);
        projects.push(gitlab_project);
    }

    fpm::utils::PagedResponse::<GitLabProject> {
        results: projects,
        token: request.token,
        next_page_url: next_page_url,
    }
}
