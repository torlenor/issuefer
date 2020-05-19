use crate::config;
use crate::issueapi::{Issue, IssueAPI};

use std::fmt;

extern crate serde_derive;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GitLabIssue {
    pub id: i64,
    pub iid: i64,
    #[serde(rename = "project_id")]
    pub project_id: i64,
    pub title: String,
    pub description: Option<String>,
    pub state: String,
    #[serde(rename = "created_at")]
    pub created_at: String,
    #[serde(rename = "updated_at")]
    pub updated_at: String,
    #[serde(rename = "closed_at")]
    pub closed_at: ::serde_json::Value,
    #[serde(rename = "closed_by")]
    pub closed_by: ::serde_json::Value,
    pub labels: Vec<::serde_json::Value>,
    pub milestone: ::serde_json::Value,
    pub assignees: Vec<::serde_json::Value>,
    pub author: Author,
    pub assignee: ::serde_json::Value,
    #[serde(rename = "user_notes_count")]
    pub user_notes_count: i64,
    #[serde(rename = "merge_requests_count")]
    pub merge_requests_count: i64,
    pub upvotes: i64,
    pub downvotes: i64,
    #[serde(rename = "due_date")]
    pub due_date: ::serde_json::Value,
    pub confidential: bool,
    #[serde(rename = "discussion_locked")]
    pub discussion_locked: ::serde_json::Value,
    #[serde(rename = "web_url")]
    pub web_url: String,
    #[serde(rename = "time_stats")]
    pub time_stats: TimeStats,
    #[serde(rename = "task_completion_status")]
    pub task_completion_status: TaskCompletionStatus,
    #[serde(rename = "has_tasks")]
    pub has_tasks: bool,
    #[serde(rename = "_links")]
    pub links: Links,
    pub references: References,
    #[serde(rename = "moved_to_id")]
    pub moved_to_id: ::serde_json::Value,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Author {
    pub id: i64,
    pub name: String,
    pub username: String,
    pub state: String,
    #[serde(rename = "avatar_url")]
    pub avatar_url: String,
    #[serde(rename = "web_url")]
    pub web_url: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TimeStats {
    #[serde(rename = "time_estimate")]
    pub time_estimate: i64,
    #[serde(rename = "total_time_spent")]
    pub total_time_spent: i64,
    #[serde(rename = "human_time_estimate")]
    pub human_time_estimate: ::serde_json::Value,
    #[serde(rename = "human_total_time_spent")]
    pub human_total_time_spent: ::serde_json::Value,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TaskCompletionStatus {
    pub count: i64,
    #[serde(rename = "completed_count")]
    pub completed_count: i64,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Links {
    #[serde(rename = "self")]
    pub self_field: String,
    pub notes: String,
    #[serde(rename = "award_emoji")]
    pub award_emoji: String,
    pub project: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct References {
    pub short: String,
    pub relative: String,
    pub full: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CreatedIssue {
    pub id: i64,
    pub iid: i64,
    #[serde(rename = "project_id")]
    pub project_id: i64,
    pub title: String,
    pub description: ::serde_json::Value,
    pub state: String,
    #[serde(rename = "created_at")]
    pub created_at: String,
    #[serde(rename = "updated_at")]
    pub updated_at: String,
    #[serde(rename = "closed_at")]
    pub closed_at: ::serde_json::Value,
    #[serde(rename = "closed_by")]
    pub closed_by: ::serde_json::Value,
    pub labels: Vec<::serde_json::Value>,
    pub milestone: ::serde_json::Value,
    pub assignees: Vec<::serde_json::Value>,
    pub author: Author,
    pub assignee: ::serde_json::Value,
    #[serde(rename = "user_notes_count")]
    pub user_notes_count: i64,
    #[serde(rename = "merge_requests_count")]
    pub merge_requests_count: i64,
    pub upvotes: i64,
    pub downvotes: i64,
    #[serde(rename = "due_date")]
    pub due_date: ::serde_json::Value,
    pub confidential: bool,
    #[serde(rename = "discussion_locked")]
    pub discussion_locked: ::serde_json::Value,
    #[serde(rename = "web_url")]
    pub web_url: String,
    #[serde(rename = "time_stats")]
    pub time_stats: TimeStats,
    #[serde(rename = "task_completion_status")]
    pub task_completion_status: TaskCompletionStatus,
    #[serde(rename = "has_tasks")]
    pub has_tasks: bool,
    #[serde(rename = "_links")]
    pub links: Links,
    pub references: References,
    pub subscribed: bool,
    #[serde(rename = "moved_to_id")]
    pub moved_to_id: ::serde_json::Value,
}

pub struct GitLabAPI {
    config: config::GitLabConfig,
    owner: String,
    repo: String,
}

impl GitLabAPI {
    pub fn new(config: config::GitLabConfig, owner: String, repo: String) -> GitLabAPI {
        GitLabAPI {
            config,
            owner,
            repo,
        }
    }
}

impl fmt::Display for GitLabAPI {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "GitLab Project {}/{}", self.owner, self.repo)
    }
}

impl IssueAPI for GitLabAPI {
    fn repo(&self) -> String {
        format!("GitLab {}/{}", self.owner, self.repo)
    }
    fn get_closed_issues(&self) -> Option<Vec<Issue>> {
        self.get_issues()
    }
    fn get_issues(&self) -> Option<Vec<Issue>> {
        // TODO (#21): Implement proper error handling when getting issues from GitLab
        // TODO: Support fetching additional pages of issues from GitLab

        let request_url = format!(
            "https://{host}/api/v4/projects/{owner}%2F{repo}/issues",
            host = self.config.host,
            owner = self.owner,
            repo = self.repo
        );
        let client = reqwest::blocking::Client::new();
        let resp = client
            .get(&request_url)
            .header("PRIVATE-TOKEN", &self.config.token)
            .header(reqwest::header::USER_AGENT, "hyper/0.5.2")
            .send()
            .unwrap();
        if resp.status().is_success() {
            let text = resp.text().unwrap();
            let deserialized: Vec<GitLabIssue> = serde_json::from_str(&text).unwrap();
            let mut issues = Vec::new();
            for gitlab_issue in deserialized {
                let issue = Issue {
                    number: gitlab_issue.iid,
                    title: gitlab_issue.title,
                    state: gitlab_issue.state,
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
        // TODO: Implement proper error handling when creating GitLab issues
        let request_url = format!(
            "https://{host}/api/v4/projects/{owner}%2F{repo}/issues",
            host = self.config.host,
            owner = self.owner,
            repo = self.repo
        );
        let resp = reqwest::blocking::Client::new()
            .post(&request_url)
            .header("PRIVATE-TOKEN", &self.config.token)
            .header(reqwest::header::USER_AGENT, "hyper/0.5.2")
            .query(&[("title", title)])
            .send()
            .unwrap();
        if resp.status().is_success() {
            let gitlab_issue: CreatedIssue = resp.json().unwrap();
            let issue = Issue {
                number: gitlab_issue.iid,
                title: gitlab_issue.title,
                state: gitlab_issue.state,
            };
            return Some(issue);
        } else if resp.status().is_server_error() {
            println!("server error!");
        } else {
            println!(
                "Something else happened when creating issue with the url '{}'. Status: {:?}",
                request_url,
                resp.status()
            );
        }
        None
    }
}
