#[macro_use]
extern crate lazy_static;
extern crate clap;
extern crate regex;
extern crate reqwest;

use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::{env, io};

use clap::{App, Arg};
use regex::Regex;

mod todo;
use crate::todo::Todo;

mod github;
mod gitlab;
mod iniparser;

pub mod issueapi;
use issueapi::{Issue, IssueAPI};

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
    // TODO (#1): Implement proper error handling when parsing source files
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

fn parse_git_config(url: &str, domain: &str) -> Result<(String, String), String> {
    let re: Regex;
    if url.starts_with("ssh://") {
        re = Regex::new(&format!(r"ssh://git@{}:?\d*/(\S+)/(\S+)\.git", domain)).unwrap();
    } else if url.starts_with("https://") {
        re = Regex::new(&format!(r"https://{}:?\d*/(\S+)/(\S+)\.git", domain)).unwrap();
    } else {
        re = Regex::new(&format!(r"git@{}:(\S+)/(\S+)\.git", domain)).unwrap();
    }

    if let Some(x) = re.captures(url) {
        return Ok((
            x.get(1).map_or("", |m| m.as_str()).to_string(),
            x.get(2).map_or("", |m| m.as_str()).to_string(),
        ));
    }

    Err("Could not extract origin URL".to_string())
}

fn get_git_config_origin_owner_repo(host: &str) -> Result<(String, String), String> {
    let current_dir = env::current_dir();
    if current_dir.is_ok() {
        let path = format!("{}/.git/config", current_dir.unwrap().to_str().unwrap());
        if !std::path::Path::new(&path).exists() {
            return Err(format!(
                "Could ot open git config {}: Path does not exist",
                path
            ));
        }

        match iniparser::parse_ini_file(&path) {
            Ok(ini) => {
                if let Ok(section) = ini.section("remote \"origin\"") {
                    if let Ok(url) = section.get("url") {
                        parse_git_config(url, host)
                    } else {
                        Err("The git repo origin remote url does not exist.".to_string())
                    }
                } else {
                    Err("The git repo does not have an origin remote.".to_string())
                }
            }
            Err(e) => Err(e),
        }
    } else {
        Err(format!(
            "Cannot determine current directory: {}",
            current_dir.err().unwrap()
        ))
    }
}

fn get_github_git_config() -> Result<(String, String), String> {
    get_git_config_origin_owner_repo("github.com")
}

fn get_gitlab_host_from_env() -> String {
    if env::var("GITLAB_HOST").is_err() {
        return "gitlab.com".to_string();
    }
    env::var("GITLAB_HOST").unwrap()
}

fn get_gitlab_git_config() -> Result<(String, String), String> {
    get_git_config_origin_owner_repo(&get_gitlab_host_from_env())
}

fn get_project_api_from_git_config() -> Result<Box<dyn IssueAPI>, String> {
    let github_config = get_github_git_config();
    if github_config.is_ok() {
        let (owner, repo) = github_config.ok().unwrap();
        return Ok(Box::new(github::GitHubAPI::new(owner, repo)));
    }
    let gitlab_config = get_gitlab_git_config();
    if gitlab_config.is_ok() {
        let (owner, repo) = gitlab_config.ok().unwrap();
        return Ok(Box::new(gitlab::GitLabAPI::new(owner, repo)));
    } else {
        println!("{:?}", gitlab_config.err());
    }
    Err("No valid GitHub or GitLab remote origin found. In case a private GitLab repository make sure GITLAB_HOST env variable is set correctly.".to_string())
}

// find_issue_by_title searches a list of issues by title and returns true if it finds an issue.
fn find_issue_by_title(issues: &[Issue], title: &str) -> bool {
    if let Some(_issue) = issues.iter().find(|&x| x.title == title) {
        return true;
    }
    false
}

// find_issue_by_number searches a list of issues by issue number and returns a copy if it finds it.
fn find_issue_by_number(issues: &[Issue], number: i64) -> Option<Issue> {
    if let Some(issue) = issues.iter().find(|&x| x.number == number) {
        return Some(issue.clone());
    }
    None
}

fn compare_todos_and_issues(todos: &[Todo], issues: &[Issue]) -> Vec<Todo> {
    let mut todos_to_create: Vec<Todo> = Vec::new();

    for todo in todos {
        if todo.issue_number == 0 && !find_issue_by_title(issues, &todo.title) {
            todos_to_create.push(todo.clone());
        }
    }

    todos_to_create
}

fn commit(file_path: &str, message: &str) {
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
            .args(&["commit", "-m", message])
            .output();
        match output {
            Ok(_v) => {}
            Err(e) => println!("Error when executing git commit: {:?}", e),
        }
    }
}

fn commit_add(file_path: &str, issue_number: i64) {
    commit(file_path, &format!("Add TODO #{}", issue_number));
}

fn commit_delete(file_path: &str, issue_number: u16) {
    commit(file_path, &format!("Remove TODO #{}", issue_number));
}

fn update_file(todo: &Todo, issue_number: i64, delete: bool) -> Result<(), io::Error> {
    let output_file_path = format!("{}.issufer", &todo.file_path);
    {
        let input_file = File::open(&todo.file_path)?;
        let reader = BufReader::new(input_file);
        let output_file = File::create(&output_file_path)?;
        let mut writer = BufWriter::new(output_file);
        for (cnt, line) in reader.lines().enumerate() {
            if cnt == todo.line_number {
                if delete {
                    continue;
                } else {
                    let new_line =
                        line?.replace("// TODO:", &format!("// TODO (#{}):", issue_number));
                    writeln!(writer, "{}", new_line)?;
                }
            } else {
                writeln!(writer, "{}", line?)?;
            }
        }
    }

    std::fs::rename(&output_file_path, &todo.file_path)?;

    Ok(())
}

fn create_github_issues_from_todos(
    api: Box<dyn IssueAPI>,
    todos_to_create: &[Todo],
    force_yes: bool,
) {
    if todos_to_create.is_empty() {
        return;
    }
    println!("Found the following unreported TODOs:");
    for todo in todos_to_create {
        println!("{}", todo);
        if force_yes || ask_yes_no("Do you want to report this TODO?") {
            if let Some(new_issue) = api.create_issue(&todo.title) {
                update_file(&todo, new_issue.number, false).unwrap();
                commit_add(&todo.file_path, new_issue.number);
                println!(
                    "Issue #{} with title '{}' created successfully",
                    new_issue.number, new_issue.title
                );
            } else {
                println!("Could not create new issue for '{}'", todo);
            }
        }
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
            update_file(&todo, 0, true).unwrap();
            commit_delete(&todo.file_path, todo.issue_number);
            println!(
                "Todo to issue #{} with title '{}' removed successfully",
                todo.issue_number, todo.title
            );
        }
    }
}

fn find_todos_to_cleanup(todos: &[Todo], issues: &[Issue]) -> Vec<Todo> {
    let mut todos_to_cleanup: Vec<Todo> = Vec::new();

    for todo in todos {
        if todo.issue_number > 0 {
            if let Some(issue) = find_issue_by_number(issues, todo.issue_number as i64) {
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

    let api: Box<dyn IssueAPI>;
    match get_project_api_from_git_config() {
        Ok(new_api) => {
            api = new_api;
        }
        Err(e) => {
            eprintln!("Could not get GitHub project from git config: {}", e);
            std::process::exit(1);
        }
    }

    println!("IssueFER running for {}\n", api.repo());

    let source_files = get_all_source_code_files()?;
    let source_code_todos = get_all_todos_from_source_code_files(&source_files);

    let github_issues = api.get_issues();
    if let Some(issues) = github_issues {
        if report {
            create_github_issues_from_todos(
                api,
                &compare_todos_and_issues(&source_code_todos, &issues),
                force_yes,
            );
        } else {
            println!("Found the following unreported TODOs:");
            for todo in compare_todos_and_issues(&source_code_todos, &issues) {
                println!("{}", todo);
            }
            println!("To report them run issuefer with the -r/--report flag");
        }
        println!();
        if cleanup {
            let todos_to_cleanup = find_todos_to_cleanup(&source_code_todos, &issues);
            remove_todos(&todos_to_cleanup, force_yes);
        } else {
            println!("Found the following TODOs to clean up:");
            for todo in find_todos_to_cleanup(&source_code_todos, &issues) {
                println!("{}", todo);
            }
            println!("To clean them up run issuefer with the -c/--cleanup flag");
        }
    } else {
        eprintln!("Could not fetch issues for current project");
        std::process::exit(1);
    }

    // TODO (#6): Add option to ignore TODOs, mark them with '// TODO (II):'
    // TODO (#7): Support more than just // at the beginning of the TODO lines
    // TODO (#8): C style multi-line comments with /* */ should be supported
    // TODO (#9): When encountering TODOs followed by commented lines  those lines shall be added to the body of the issue
    // TODO (#10): Make it possible to ignore files by their extension, .e.g., .md

    Ok(())
}
