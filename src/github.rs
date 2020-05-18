use crate::config;
use crate::issueapi::{Issue, IssueAPI};

use std::fmt;

extern crate serde_derive;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GitHubIssue {
    pub url: String,
    #[serde(rename = "repository_url")]
    pub repository_url: String,
    #[serde(rename = "labels_url")]
    pub labels_url: String,
    #[serde(rename = "comments_url")]
    pub comments_url: String,
    #[serde(rename = "events_url")]
    pub events_url: String,
    #[serde(rename = "html_url")]
    pub html_url: String,
    pub id: i64,
    #[serde(rename = "node_id")]
    pub node_id: String,
    pub number: i64,
    pub title: String,
    pub user: User,
    pub labels: Vec<::serde_json::Value>,
    pub state: String,
    pub locked: bool,
    pub assignee: ::serde_json::Value,
    pub assignees: Vec<::serde_json::Value>,
    pub milestone: ::serde_json::Value,
    pub comments: i64,
    #[serde(rename = "created_at")]
    pub created_at: String,
    #[serde(rename = "updated_at")]
    pub updated_at: String,
    #[serde(rename = "closed_at")]
    pub closed_at: ::serde_json::Value,
    #[serde(rename = "author_association")]
    pub author_association: String,
    pub body: ::serde_json::Value,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub login: String,
    pub id: i64,
    #[serde(rename = "node_id")]
    pub node_id: String,
    #[serde(rename = "avatar_url")]
    pub avatar_url: String,
    #[serde(rename = "gravatar_id")]
    pub gravatar_id: String,
    pub url: String,
    #[serde(rename = "html_url")]
    pub html_url: String,
    #[serde(rename = "followers_url")]
    pub followers_url: String,
    #[serde(rename = "following_url")]
    pub following_url: String,
    #[serde(rename = "gists_url")]
    pub gists_url: String,
    #[serde(rename = "starred_url")]
    pub starred_url: String,
    #[serde(rename = "subscriptions_url")]
    pub subscriptions_url: String,
    #[serde(rename = "organizations_url")]
    pub organizations_url: String,
    #[serde(rename = "repos_url")]
    pub repos_url: String,
    #[serde(rename = "events_url")]
    pub events_url: String,
    #[serde(rename = "received_events_url")]
    pub received_events_url: String,
    #[serde(rename = "type")]
    pub type_field: String,
    #[serde(rename = "site_admin")]
    pub site_admin: bool,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CreatedIssue {
    pub url: String,
    #[serde(rename = "repository_url")]
    pub repository_url: String,
    #[serde(rename = "labels_url")]
    pub labels_url: String,
    #[serde(rename = "comments_url")]
    pub comments_url: String,
    #[serde(rename = "events_url")]
    pub events_url: String,
    #[serde(rename = "html_url")]
    pub html_url: String,
    pub id: i64,
    #[serde(rename = "node_id")]
    pub node_id: String,
    pub number: i64,
    pub title: String,
    pub user: User,
    pub labels: Vec<::serde_json::Value>,
    pub state: String,
    pub locked: bool,
    pub assignee: ::serde_json::Value,
    pub assignees: Vec<::serde_json::Value>,
    pub milestone: ::serde_json::Value,
    pub comments: i64,
    #[serde(rename = "created_at")]
    pub created_at: String,
    #[serde(rename = "updated_at")]
    pub updated_at: String,
    #[serde(rename = "closed_at")]
    pub closed_at: ::serde_json::Value,
    #[serde(rename = "author_association")]
    pub author_association: String,
    pub body: ::serde_json::Value,
    #[serde(rename = "closed_by")]
    pub closed_by: ::serde_json::Value,
}

pub struct GitHubAPI {
    config: config::GitHubConfig,
    owner: String,
    repo: String,
}

impl GitHubAPI {
    // Another static method, taking two arguments:
    pub fn new(config: config::GitHubConfig, owner: String, repo: String) -> GitHubAPI {
        GitHubAPI {
            config,
            owner,
            repo,
        }
    }
}

impl fmt::Display for GitHubAPI {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "GitHub Project {}/{}", self.owner, self.repo)
    }
}

impl IssueAPI for GitHubAPI {
    fn repo(&self) -> String {
        format!("GitHub {}/{}", self.owner, self.repo)
    }
    fn get_issues(&self) -> Option<Vec<Issue>> {
        // Doc: https://developer.github.com/v3/issues/#get-an-issue
        // TODO (#3): Implement proper error handling when getting issues from GitHub
        // TODO (#4): Support fetching additional pages of issues from GitHub
        let request_url = format!(
            "https://api.github.com/repos/{owner}/{repo}/issues?state=all",
            owner = self.owner,
            repo = self.repo
        );
        let client = reqwest::blocking::Client::new();
        let resp = client
            .get(&request_url)
            .header(
                reqwest::header::AUTHORIZATION,
                format!("token {token}", token = self.config.token),
            )
            .header(reqwest::header::USER_AGENT, "hyper/0.5.2")
            .send()
            .unwrap();
        if resp.status().is_success() {
            let text = resp.text().unwrap();
            let deserialized: Vec<GitHubIssue> = serde_json::from_str(&text).unwrap();
            let mut issues = Vec::new();
            for github_issue in deserialized {
                let issue = Issue {
                    number: github_issue.number,
                    title: github_issue.title,
                    state: github_issue.state,
                };
                issues.push(issue);
            }
            return Some(issues);
        } else if resp.status().is_server_error() {
            println!("server error!");
        } else {
            println!(
                "Something else happened. Status: {:?}, Body: {:?}",
                resp.status(),
                resp.text()
            );
        }
        None
    }

    fn create_issue(&self, title: &str) -> Option<Issue> {
        // TODO (#5): Implement proper error handling when creating GitHub issues
        let mut issue_body = std::collections::HashMap::new();
        issue_body.insert("title", title);
        let request_url = format!(
            "https://api.github.com/repos/{owner}/{repo}/issues?state=all",
            owner = self.owner,
            repo = self.repo
        );
        let resp = reqwest::blocking::Client::new()
            .post(&request_url)
            .json(&issue_body)
            .header(
                reqwest::header::AUTHORIZATION,
                format!("token {token}", token = self.config.token),
            )
            .header(reqwest::header::USER_AGENT, "hyper/0.5.2")
            .send()
            .unwrap();
        if resp.status().is_success() {
            let github_issue: CreatedIssue = resp.json().unwrap();
            let issue = Issue {
                number: github_issue.number,
                title: github_issue.title,
                state: github_issue.state,
            };
            return Some(issue);
        } else if resp.status().is_server_error() {
            println!("server error!");
        } else {
            println!(
                "Something else happened. Status: {:?}, Body: {:?}",
                resp.status(),
                resp.text()
            );
        }
        None
    }
}
