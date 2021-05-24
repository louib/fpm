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
    let mut response: Vec<GitHubRepo> = vec![];

    log::info!("Search GitHub for term {}.", search_term);
    let base_search_url = "https://api.github.com/search/repositories?type=all&per_page=100";

    // The GitHub search API limits the number of search results to 1000, so
    // we need to split the search request to make sure we can access the search results.

    let projects = search_repos_internal(&format!(
        "{}&q={}+in:readme fork:false -org:flathub created:2012-01-01..2017-01-01",
        base_search_url, search_term,
    ));
    for project in projects {
        response.push(project);
    }

    let projects = search_repos_internal(&format!(
        "{}&q={}+in:readme fork:false -org:flathub created:2017-01-01..2019-01-01",
        base_search_url, search_term,
    ));
    for project in projects {
        response.push(project);
    }

    let projects = search_repos_internal(&format!(
        "{}&q={}+in:readme fork:false -org:flathub created:2019-01-01..2020-01-01",
        base_search_url, search_term,
    ));
    for project in projects {
        response.push(project);
    }

    let projects = search_repos_internal(&format!(
        "{}&q={}+in:readme fork:false -org:flathub created:2020-01-01..2021-01-01",
        base_search_url, search_term,
    ));
    for project in projects {
        response.push(project);
    }

    let projects = search_repos_internal(&format!(
        "{}&q={}+in:readme fork:false -org:flathub created:>2021-01-01",
        base_search_url, search_term,
    ));
    for project in projects {
        response.push(project);
    }

    let projects = search_repos_internal(&format!(
        "{}&q=topic:{} fork:false -org:flathub",
        base_search_url, search_term,
    ));
    for project in projects {
        response.push(project);
    }

    log::info!(
        "A total of {} were returned when search GitHub for term {}.",
        response.len(),
        search_term
    );
    return response;
}

pub fn search_repos_internal(search_url: &str) -> Vec<GitHubRepo> {
    let mut projects: Vec<GitHubRepo> = vec![];

    let client = get_github_client();
    let mut next_page_url = search_url.to_string();

    while !next_page_url.is_empty() {
        log::info!("Calling GitHub API at {}", &next_page_url);
        // TODO make this really asynchronous with async/await.
        let response = match client.get(&next_page_url).send() {
            Ok(r) => r,
            Err(e) => {
                log::error!("Could not fetch GitHub url {}: {}.", &next_page_url, e);
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
        next_page_url = match fpm::utils::get_next_page_url(link_header) {
            Some(u) => u,
            None => "".to_string(),
        };

        let response_content = response.text().unwrap();
        let response: GitHubRepoSearchResponse = match serde_yaml::from_str(&response_content) {
            Ok(p) => p,
            Err(e) => {
                log::error!(
                    "Could not parse GitHub repo search response {}: {}.",
                    e,
                    &response_content
                );
                return projects;
            }
        };
        log::info!("Number of results for search is {}", response.total_count);
        if response.total_count > 1000 {
            log::error!(
                "Number of results is > 1000 ({}). Please refine you search.",
                response.total_count
            );
            return projects;
        }

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

pub fn get_org_repos(org_name: &str) -> Vec<GitHubRepo> {
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

pub fn get_repos(request: fpm::utils::PagedRequest) -> fpm::utils::PagedResponse<GitHubRepo> {
    // By default, we get all the repos.
    let mut current_url = format!("https://api.github.com/repositories?type=all&per_page=2");
    if let Some(url) = request.next_page_url {
        current_url = url;
    }

    let mut projects: Vec<GitHubRepo> = vec![];
    let default_response = fpm::utils::PagedResponse::<GitHubRepo> {
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
        projects.push(github_project);
    }

    fpm::utils::PagedResponse::<GitHubRepo> {
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
