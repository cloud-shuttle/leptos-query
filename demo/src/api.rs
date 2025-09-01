use crate::types::*;
use reqwest::Client;
use std::collections::HashMap;

// GitHub API client
pub struct GitHubApi {
    client: Client,
    base_url: String,
}

impl GitHubApi {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            base_url: "https://api.github.com".to_string(),
        }
    }

    // Fetch repository information
    pub async fn get_repository(&self, owner: &str, repo: &str) -> Result<Repository, String> {
        let url = format!("{}/repos/{}/{}", self.base_url, owner, repo);
        
        let response = self.client
            .get(&url)
            .header("Accept", "application/vnd.github.v3+json")
            .header("User-Agent", "leptos-query-demo")
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("GitHub API error: {}", response.status()));
        }

        response.json::<Repository>()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))
    }

    // Fetch issues for a repository
    pub async fn get_issues(
        &self,
        owner: &str,
        repo: &str,
        filters: &IssueFilters,
    ) -> Result<Vec<Issue>, String> {
        let mut params = HashMap::new();
        params.insert("state", &filters.state);
        params.insert("sort", &filters.sort);
        params.insert("direction", &filters.direction);
        params.insert("per_page", &"30".to_string());

        if let Some(assignee) = &filters.assignee {
            params.insert("assignee", assignee);
        }

        if let Some(creator) = &filters.creator {
            params.insert("creator", creator);
        }

        if !filters.labels.is_empty() {
            params.insert("labels", &filters.labels.join(","));
        }

        let url = format!("{}/repos/{}/{}/issues", self.base_url, owner, repo);
        
        let response = self.client
            .get(&url)
            .query(&params)
            .header("Accept", "application/vnd.github.v3+json")
            .header("User-Agent", "leptos-query-demo")
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("GitHub API error: {}", response.status()));
        }

        response.json::<Vec<Issue>>()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))
    }

    // Fetch a specific issue
    pub async fn get_issue(
        &self,
        owner: &str,
        repo: &str,
        issue_number: u32,
    ) -> Result<Issue, String> {
        let url = format!("{}/repos/{}/{}/issues/{}", self.base_url, owner, repo, issue_number);
        
        let response = self.client
            .get(&url)
            .header("Accept", "application/vnd.github.v3+json")
            .header("User-Agent", "leptos-query-demo")
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("GitHub API error: {}", response.status()));
        }

        response.json::<Issue>()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))
    }

    // Fetch comments for an issue
    pub async fn get_issue_comments(
        &self,
        owner: &str,
        repo: &str,
        issue_number: u32,
    ) -> Result<Vec<Comment>, String> {
        let url = format!("{}/repos/{}/{}/issues/{}/comments", self.base_url, owner, repo, issue_number);
        
        let response = self.client
            .get(&url)
            .header("Accept", "application/vnd.github.v3+json")
            .header("User-Agent", "leptos-query-demo")
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("GitHub API error: {}", response.status()));
        }

        response.json::<Vec<Comment>>()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))
    }

    // Search repositories
    pub async fn search_repositories(&self, query: &str) -> Result<SearchResult<Repository>, String> {
        let url = format!("{}/search/repositories", self.base_url);
        
        let mut params = HashMap::new();
        params.insert("q", query);
        params.insert("sort", "stars");
        params.insert("order", "desc");
        params.insert("per_page", "10");

        let response = self.client
            .get(&url)
            .query(&params)
            .header("Accept", "application/vnd.github.v3+json")
            .header("User-Agent", "leptos-query-demo")
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("GitHub API error: {}", response.status()));
        }

        response.json::<SearchResult<Repository>>()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))
    }
}

// Global API instance
pub fn get_api() -> GitHubApi {
    GitHubApi::new()
}
