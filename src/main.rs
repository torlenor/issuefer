extern crate ini;
extern crate regex;
extern crate reqwest;
extern crate serde_derive;
extern crate serde_json;
#[macro_use]
extern crate lazy_static;

extern crate walkdir;

use serde::{Deserialize, Serialize};
use std::error::Error;
use std::{env, io};

use std::fs::File;
use std::io::{BufRead, BufReader};

use ini::Ini;
use regex::Regex;
use walkdir::WalkDir;

#[derive(Debug)]
struct Todo {
    title: String,
    issue_number: u16,
}

#[derive(Debug)]
struct SourceCodeFile {
    file_path: String,
    todos: Vec<(usize, Todo)>, // [line_number, Todo]
}

fn get_all_source_code_files() -> Result<Vec<String>, io::Error> {
    // TODO: It shall ignore files and directories from .gitignore

    let mut source_files: Vec<String> = Vec::new();

    let current_dir = env::current_dir()?;

    for entry in WalkDir::new(current_dir)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let f_path = entry.path().to_string_lossy();
        let _sec = entry.metadata()?.modified()?;

        if f_path.ends_with(".rs") || f_path.ends_with(".go") {
            source_files.push(f_path.to_string());
        }
    }

    Ok(source_files)
}

fn parse_line(line: &str) -> Option<Todo> {
    lazy_static! {
        static ref TODO_RE: Regex = Regex::new(r"^\s*//\s+TODO:\s+(.*)$").unwrap();
        static ref TODO_SEEN_RE: Regex = Regex::new(r"^\s*//\s+TODO \(#(\d+)\):\s+(.*)$").unwrap();
    }

    if let Some(x) = TODO_RE.captures(line) {
        let t = Todo {
            title: x.get(1).map_or("", |m| m.as_str()).to_string(),
            issue_number: 0,
        };
        return Some(t);
    }

    if let Some(x) = TODO_SEEN_RE.captures(line) {
        let issue_number = x
            .get(1)
            .map_or("0", |m| m.as_str())
            .to_string()
            .parse::<u16>()
            .unwrap();
        let t = Todo {
            title: x.get(2).map_or("", |m| m.as_str()).to_string(),
            issue_number,
        };
        return Some(t);
    }
    None
}

fn get_todos_from_source_code_file(source_file: &str) -> SourceCodeFile {
    // TODO: Handle error better
    let f = File::open(source_file).expect("Unable to open file");
    let f = BufReader::new(f);

    let mut file = SourceCodeFile {
        file_path: source_file.to_string(),
        todos: Vec::new(),
    };
    for (cnt, line) in f.lines().enumerate() {
        let line = line.expect("Unable to read line");
        let result = parse_line(&line);
        if let Some(x) = result {
            file.todos.push((cnt, x))
        }
    }

    file
}

fn get_all_todos_from_source_code_files(source_files: &[String]) -> Vec<SourceCodeFile> {
    let mut source_code_todos = Vec::new();
    for source_file in source_files {
        let source_file_todos = get_todos_from_source_code_file(source_file);
        source_code_todos.push(source_file_todos);
    }
    source_code_todos
}

fn parse_git_config(url: &str) -> Option<(String, String)> {
    let re = Regex::new(r"git@github.com:(\S+)/(\S+)\.git").unwrap();

    if let Some(x) = re.captures(url) {
        return Some((
            x.get(1).map_or("", |m| m.as_str()).to_string(),
            x.get(2).map_or("", |m| m.as_str()).to_string(),
        ));
    }

    None
}

fn get_current_project_from_git_config() -> Option<(String, String)> {
    // TODO: implement proper error handling
    // TODO: The used ini parser dies if it encounters a line starting with #, i.e., a comment
    let current_dir = env::current_dir().unwrap();
    let conf =
        Ini::load_from_file(format!("{}/.git/config", current_dir.to_str().unwrap())).unwrap();

    let section = conf.section(Some("remote \"origin\"")).unwrap();
    let url = section.get("url").unwrap();

    parse_git_config(url)
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GitHubRepositoryIssues {
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
    pub body: String,
}

#[derive(Serialize, Deserialize, Debug)]
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

fn get_issues_from_github(token: &str, owner: &str, repo: &str) -> Vec<GitHubRepositoryIssues> {
    // Doc: https://developer.github.com/v3/issues/#get-an-issue
    // TODO: Implement correct error handling
    // TODO: Implement support for pages in response
    let request_url = format!(
        "https://api.github.com/repos/{owner}/{repo}/issues?state=all",
        owner = owner,
        repo = repo
    );
    let client = reqwest::blocking::Client::new();
    let response = client
        .get(&request_url)
        .header(
            reqwest::header::AUTHORIZATION,
            format!("token {token}", token = token),
        )
        .header(reqwest::header::USER_AGENT, "hyper/0.5.2")
        .send()
        .unwrap();

    let root: Vec<GitHubRepositoryIssues> = response.json().unwrap();
    root
}

fn get_token_from_env() -> Option<String> {
    if env::var("GITHUB_TOKEN").is_err() {
        return None;
    }
    Some(env::var("GITHUB_TOKEN").unwrap())
}

fn fetch_current_github_issues() -> Option<Vec<GitHubRepositoryIssues>> {
    get_current_project_from_git_config();

    if let Some(x) = get_token_from_env() {
        if let Some((owner, repo)) = get_current_project_from_git_config() {
            Some(get_issues_from_github(&x, &owner, &repo))
        } else {
            None
        }
    } else {
        println!("No GitHub token specified. Use env variable GITHUB_TOKEN to provide one.");
        None
    }
}

fn print_todos(source_code_todos: &[SourceCodeFile]) {
    println!("Found the following TODOs for the current project:");
    for file in source_code_todos {
        for todo in &file.todos {
            if todo.1.issue_number > 0 {
                println!(
                    "{}:{}: Tracked TODO {}: {}",
                    file.file_path,
                    todo.0 + 1,
                    todo.1.issue_number,
                    todo.1.title
                )
            } else {
                println!(
                    "{}:{}: Untracked TODO: {}",
                    file.file_path,
                    todo.0 + 1,
                    todo.1.title
                )
            }
        }
    }
}

fn print_github_issues(github_issues: &[GitHubRepositoryIssues]) {
    println!("\nFound the following GitHub issues for the current project:");
    for issue in github_issues {
        println!("#{} {} ({})", issue.number, issue.title, issue.state);
    }
}

fn find_github_issue_by_title(github_issues: &[GitHubRepositoryIssues], title: &str) {
    println!(
        "Find in issues by title: {:?}",
        github_issues.iter().find(|&x| x.title == title)
    );
}

fn find_github_issue_by_number(github_issues: &[GitHubRepositoryIssues], number: &u16) {
    println!(
        "Find 2 in issues by number: {:?}",
        github_issues.iter().find(|&x| x.number == *number as i64)
    );
}

fn compare_todos_and_issues(
    source_code_todos: &[SourceCodeFile],
    github_issues: &[GitHubRepositoryIssues],
) {
    for file in source_code_todos {
        for (_line, todo) in &file.todos {
            if todo.issue_number > 0 {
                find_github_issue_by_number(github_issues, &todo.issue_number)
            } else {
                find_github_issue_by_title(github_issues, &todo.title)
            }
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let source_files = get_all_source_code_files()?;
    let source_code_todos = get_all_todos_from_source_code_files(&source_files);
    print_todos(&source_code_todos);

    let github_issues = fetch_current_github_issues();
    if let Some(issues) = github_issues {
        print_github_issues(&issues);
        compare_todos_and_issues(&source_code_todos, &issues);
    } else {
        println!("Could not fetch GitHub issues for current project")
    }

    // TODO: compare_todos_and_github_issues() has to be implemented
    // TODO: create_new_github_issues() has to be implemented
    // TODO (#123): update_source_code_and_commit() has to be implemented
    // TODO: It shall be possible to ignore TODOs via CLI, maybe mark them with // TODO (II): in the file

    Ok(())
}
