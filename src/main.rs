extern crate ini;
extern crate regex;
extern crate reqwest;
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate lazy_static;

extern crate clap;
use clap::{App, Arg};

use serde::{Deserialize, Serialize};

use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::{env, fmt, io};

use ini::Ini;
use regex::Regex;

#[derive(Clone)]
struct Todo {
    file_path: String,
    line_number: usize,
    title: String,
    issue_number: u16,
}

impl fmt::Display for Todo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.issue_number == 0 {
            write!(
                f,
                "{}:{}: TODO: {}",
                self.file_path,
                self.line_number + 1,
                self.title
            )
        } else {
            write!(
                f,
                "{}:{}: TODO (#{}): {}",
                self.file_path,
                self.line_number + 1,
                self.issue_number,
                self.title
            )
        }
    }
}

fn ask_yes_no(question: &str) -> bool {
    let mut ch = ' ';
    while ch != 'y' && ch != 'n' {
        println!("{} [y/n]", question);
        let mut line = String::new();
        if io::stdin().read_line(&mut line).is_ok() {
            line = line.trim().to_string();
            if let Some(c) = line.chars().next() {
                ch = c;
            }
        }
    }
    if ch == 'y' {
        return true;
    }

    false
}

fn get_all_source_code_files() -> Result<Vec<String>, io::Error> {
    let mut source_files: Vec<String> = Vec::new();

    let current_dir = env::current_dir()?;

    {
        let output = std::process::Command::new("git")
            .args(&["ls-files"])
            .output();
        match output {
            Ok(_v) => {
                if _v.status.success() {
                    for line in String::from_utf8(_v.stdout).unwrap().lines() {
                        source_files.push(format!(
                            "{}/{}",
                            current_dir.to_str().unwrap(),
                            line.to_string()
                        ));
                    }
                }
            }
            Err(e) => println!("Error when executing git ls-files: {:?}", e),
        }
    }

    Ok(source_files)
}

fn parse_line(file_path: &str, line_number: usize, line: &str) -> Option<Todo> {
    lazy_static! {
        static ref TODO_RE: Regex = Regex::new(r"^\s*//\s+TODO:\s+(.*)$").unwrap();
        static ref TODO_SEEN_RE: Regex = Regex::new(r"^\s*//\s+TODO \(#(\d+)\):\s+(.*)$").unwrap();
    }

    if let Some(x) = TODO_RE.captures(line) {
        let t = Todo {
            file_path: file_path.to_string(),
            line_number,
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
            file_path: file_path.to_string(),
            line_number,
            title: x.get(2).map_or("", |m| m.as_str()).to_string(),
            issue_number,
        };
        return Some(t);
    }
    None
}

fn get_todos_from_source_code_file(source_file: &str) -> Vec<Todo> {
    // TODO: Handle errors when parsing source files for TODOs
    let f = File::open(source_file).expect("Unable to open file");
    let f = BufReader::new(f);

    let mut todos = Vec::new();

    for (cnt, line) in f.lines().enumerate() {
        let line = line.expect("Unable to read line");
        let result = parse_line(source_file, cnt, &line);
        if let Some(todo) = result {
            todos.push(todo)
        }
    }

    todos
}

fn get_all_todos_from_source_code_files(source_files: &[String]) -> Vec<Todo> {
    let mut all_todos = Vec::new();
    for source_file in source_files {
        let todos = get_todos_from_source_code_file(source_file);
        all_todos.extend(todos);
    }
    all_todos
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
    // TODO: Implement proper error handling when reading git config
    // TODO: The used ini parser dies if it encounters a line starting with #, i.e., a comment
    let current_dir = env::current_dir();
    if current_dir.is_ok() {
        let path = format!("{}/.git/config", current_dir.unwrap().to_str().unwrap());
        if !std::path::Path::new(&path).exists() {
            return None;
        }

        let conf = Ini::load_from_file(path).unwrap();

        let section = conf.section(Some("remote \"origin\"")).unwrap();
        let url = section.get("url").unwrap();
        parse_git_config(url)
    } else {
        None
    }
}

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

fn get_issues_from_github(token: &str, owner: &str, repo: &str) -> Option<Vec<GitHubIssue>> {
    // Doc: https://developer.github.com/v3/issues/#get-an-issue
    // TODO: Implement correct error handling when getting issues from GitHub
    // TODO: Implement support for pages in response when retrieving GitHub issues
    let request_url = format!(
        "https://api.github.com/repos/{owner}/{repo}/issues?state=all",
        owner = owner,
        repo = repo
    );
    let client = reqwest::blocking::Client::new();
    let resp = client
        .get(&request_url)
        .header(
            reqwest::header::AUTHORIZATION,
            format!("token {token}", token = token),
        )
        .header(reqwest::header::USER_AGENT, "hyper/0.5.2")
        .send()
        .unwrap();

    if resp.status().is_success() {
        let text = resp.text().unwrap();
        let deserialized: Vec<GitHubIssue> = serde_json::from_str(&text).unwrap();
        return Some(deserialized);
    } else if resp.status().is_server_error() {
        println!("server error!");
    } else {
        println!("Something else happened. Status: {:?}", resp.status());
    }

    None
}

fn get_token_from_env() -> Option<String> {
    if env::var("GITHUB_TOKEN").is_err() {
        return None;
    }
    Some(env::var("GITHUB_TOKEN").unwrap())
}

fn fetch_current_github_issues() -> Option<Vec<GitHubIssue>> {
    if let Some(x) = get_token_from_env() {
        if let Some((owner, repo)) = get_current_project_from_git_config() {
            get_issues_from_github(&x, &owner, &repo)
        } else {
            println!("Could not get GitHub project from git config. Make sure we are in a git repository and has GitHub as origin.");
            None
        }
    } else {
        println!("No GitHub token specified. Use env variable GITHUB_TOKEN to provide one.");
        None
    }
}

// find_github_issue_by_title searches a list of GitHub issues by title and returns true if it finds an issue.
fn find_github_issue_by_title(github_issues: &[GitHubIssue], title: &str) -> bool {
    if let Some(_issue) = github_issues.iter().find(|&x| x.title == title) {
        return true;
    }
    false
}

// find_github_issue_by_number searches a list of GitHub issues by issue number and returns a copy if it finds it.
fn find_github_issue_by_number(github_issues: &[GitHubIssue], number: i64) -> Option<GitHubIssue> {
    if let Some(issue) = github_issues.iter().find(|&x| x.number == number) {
        return Some(issue.clone());
    }
    None
}

fn compare_todos_and_issues(todos: &[Todo], github_issues: &[GitHubIssue]) -> Vec<Todo> {
    let mut todos_to_create: Vec<Todo> = Vec::new();

    for todo in todos {
        if todo.issue_number == 0 && !find_github_issue_by_title(github_issues, &todo.title) {
            todos_to_create.push(todo.clone());
        }
    }

    todos_to_create
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

fn create_github_issue(owner: &str, repo: &str, token: &str, title: &str) -> Option<CreatedIssue> {
    // TODO: The function that creates GitHub issues needs proper error handling
    let issue_body = json!({
    "title": title,
    });

    let request_url = format!(
        "https://api.github.com/repos/{owner}/{repo}/issues?state=all",
        owner = owner,
        repo = repo
    );
    let resp = reqwest::blocking::Client::new()
        .post(&request_url)
        .json(&issue_body)
        .header(
            reqwest::header::AUTHORIZATION,
            format!("token {token}", token = token),
        )
        .header(reqwest::header::USER_AGENT, "hyper/0.5.2")
        .send()
        .unwrap();

    if resp.status().is_success() {
        let issue: CreatedIssue = resp.json().unwrap();
        return Some(issue);
    } else if resp.status().is_server_error() {
        println!("server error!");
    } else {
        println!("Something else happened. Status: {:?}", resp.status());
    }

    None
}

fn commit(file_path: &str, issue_number: i64) {
    {
        let output = std::process::Command::new("git")
            .args(&["add", file_path])
            .output();
        match output {
            Ok(_v) => {}
            Err(e) => println!("Error when executing git add: {:?}", e),
        }
    }

    {
        let output = std::process::Command::new("git")
            .args(&["commit", "-m", &format!("Add TODO #{}", issue_number)])
            .output();
        match output {
            Ok(_v) => {}
            Err(e) => println!("Error when executing git commit: {:?}", e),
        }
    }
}

fn commit_delete(file_path: &str, issue_number: u16) {
    {
        let output = std::process::Command::new("git")
            .args(&["add", file_path])
            .output();
        match output {
            Ok(_v) => {}
            Err(e) => println!("Error when executing git add: {:?}", e),
        }
    }

    {
        let output = std::process::Command::new("git")
            .args(&["commit", "-m", &format!("Remove TODO #{}", issue_number)])
            .output();
        match output {
            Ok(_v) => {}
            Err(e) => println!("Error when executing git commit: {:?}", e),
        }
    }
}

fn update_file(todo: &Todo, issue_number: i64) -> Result<(), io::Error> {
    let output_file_path = format!("{}.issufer", &todo.file_path);
    {
        let input_file = File::open(&todo.file_path)?;
        let reader = BufReader::new(input_file);
        let output_file = File::create(&output_file_path)?;
        let mut writer = BufWriter::new(output_file);
        for (cnt, line) in reader.lines().enumerate() {
            if cnt == todo.line_number {
                let new_line = line?.replace("// TODO:", &format!("// TODO (#{}):", issue_number));
                writeln!(writer, "{}", new_line)?;
            } else {
                writeln!(writer, "{}", line?)?;
            }
        }
    }

    std::fs::rename(&output_file_path, &todo.file_path)?;

    Ok(())
}

fn update_file_delete_todo(todo: &Todo) -> Result<(), io::Error> {
    let output_file_path = format!("{}.issufer", &todo.file_path);
    {
        let input_file = File::open(&todo.file_path)?;
        let reader = BufReader::new(input_file);
        let output_file = File::create(&output_file_path)?;
        let mut writer = BufWriter::new(output_file);
        for (cnt, line) in reader.lines().enumerate() {
            if cnt != todo.line_number {
                writeln!(writer, "{}", line?)?;
            }
        }
    }

    std::fs::rename(&output_file_path, &todo.file_path)?;

    Ok(())
}

fn create_github_issues_from_todos(todos_to_create: &[Todo], force_yes: bool) {
    if todos_to_create.is_empty() {
        return;
    }
    println!("Found the following unreported TODOs:");
    if let Some(token) = get_token_from_env() {
        if let Some((owner, repo)) = get_current_project_from_git_config() {
            for todo in todos_to_create {
                println!("{}", todo);
                if force_yes || ask_yes_no("Do you want to report this TODO?") {
                    if let Some(new_issue) = create_github_issue(&owner, &repo, &token, &todo.title)
                    {
                        update_file(&todo, new_issue.number).unwrap();
                        commit(&todo.file_path, new_issue.number);
                        println!(
                            "Issue #{} with title '{}' created successfully",
                            new_issue.number, new_issue.title
                        );
                    } else {
                        println!(
                            "Could not create new GitHub issue for TODO {}:{}: {}",
                            todo.file_path,
                            todo.line_number + 1,
                            todo.title
                        );
                    }
                }
            }
        } else {
            println!("No valid GitHub project repository found in current directory.");
        }
    } else {
        println!("No GitHub token specified. Use env variable GITHUB_TOKEN to provide one.");
    }
}

fn remove_todos(todos_to_remove: &[Todo], force_yes: bool) {
    if todos_to_remove.is_empty() {
        return;
    }
    println!("Found the following TODOs to remove:");
    for todo in todos_to_remove {
        println!("{}", todo);
        if force_yes || ask_yes_no("Do you want to remove this TODO?") {
            update_file_delete_todo(&todo).unwrap();
            commit_delete(&todo.file_path, todo.issue_number);
            println!(
                "Todo to issue #{} with title '{}' removed successfully",
                todo.issue_number, todo.title
            );
        }
    }
}

fn find_todos_to_cleanup(todos: &[Todo], github_issues: &[GitHubIssue]) -> Vec<Todo> {
    let mut todos_to_cleanup: Vec<Todo> = Vec::new();

    for todo in todos {
        if todo.issue_number > 0 {
            if let Some(issue) =
                find_github_issue_by_number(github_issues, todo.issue_number as i64)
            {
                if issue.state == "closed" {
                    todos_to_cleanup.push(todo.clone());
                }
            }
        }
    }

    todos_to_cleanup
}

fn main() -> Result<(), Box<dyn Error>> {
    let matches = App::new("IssueFER")
        .version("0.1.0")
        .author("Torlenor <torlenor@abyle.org>")
        .about("Turns TODOs into GitHub issues")
        .arg(
            Arg::with_name("report")
                .short("r")
                .long("report")
                .help("Report all newly found TODOs"),
        )
        .arg(
            Arg::with_name("cleanup")
                .short("c")
                .long("cleanup")
                .help("Cleanup issues to closed TODOs"),
        )
        .arg(
            Arg::with_name("force-yes")
                .short("y")
                .long("force-yes")
                .help("Answer every question with yes (e.g., report all TODOs as issues)"),
        )
        .get_matches();

    let report = matches.is_present("report");
    let cleanup = matches.is_present("cleanup");
    let force_yes = matches.is_present("force-yes");

    if let Some((owner, repo)) = get_current_project_from_git_config() {
        println!("IssueFER running for GitHub project '{}/{}'\n", owner, repo);
    } else {
        eprintln!("Could not get GitHub project from git config. Make sure we are in a git repository and has GitHub as origin.");
        std::process::exit(1);
    }

    let source_files = get_all_source_code_files()?;
    let source_code_todos = get_all_todos_from_source_code_files(&source_files);

    let github_issues = fetch_current_github_issues();
    if let Some(issues) = github_issues {
        if report {
            create_github_issues_from_todos(
                &compare_todos_and_issues(&source_code_todos, &issues),
                force_yes,
            );
        } else {
            println!("Found the following unreported TODOs:");
            for todo in compare_todos_and_issues(&source_code_todos, &issues) {
                println!("{}", todo);
            }
            println!("\nTo report them run issuefer with the -r/--report flag");
        }
        println!();
        if cleanup {
            let todos_to_cleanup = find_todos_to_cleanup(&source_code_todos, &issues);
            remove_todos(&todos_to_cleanup, force_yes);
        } else {
            println!("\nFound the following TODOs to clean up:");
            for todo in find_todos_to_cleanup(&source_code_todos, &issues) {
                println!("{}", todo);
            }
            println!("To clean them up run issuefer with the -c/--cleanup flag");
        }
    } else {
        eprintln!("Could not fetch GitHub issues for current project");
        std::process::exit(1);
    }

    // TODO: It shall be possible to ignore TODOs via CLI, maybe mark them with // TODO (II): in the file

    Ok(())
}
