use std::error::Error;
use std::{env, io};

use std::fs::File;
use std::io::{BufRead, BufReader};

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
    let mut source_files: Vec<String> = Vec::new();

    let current_dir = env::current_dir()?;

    for entry in WalkDir::new(current_dir)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let f_path = entry.path().to_string_lossy();
        let _sec = entry.metadata()?.modified()?;

        if f_path.ends_with(".rs") || f_path.ends_with("*.go") {
            source_files.push(f_path.to_string());
        }
    }

    Ok(source_files)
}

fn parse_line(line: &str) -> Option<Todo> {
    let re = Regex::new(r"^\s*//\s+TODO:\s+(.*)$").unwrap();
    let text = line;
    if re.is_match(text) {
        for cap in re.captures_iter(text) {
            let t = Todo {
                title: cap[1].to_string(),
                issue_number: 0,
            };
            return Some(t);
        }
        // if re.captures_len() >= 1 {
        //     re.capture
        // }
    }

    let re = Regex::new(r"^\s*//\s+TODO \(#(\d+)\):\s+(.*)$").unwrap();
    let text = line;
    if re.is_match(text) {
        for cap in re.captures_iter(text) {
            let issue_number = cap[1].to_string().parse::<u16>().unwrap();
            let t = Todo {
                title: cap[2].to_string(),
                issue_number,
            };
            return Some(t);
        }
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
    // let mut cnt: usize = 0;
    // for line in f.lines() {
    for (cnt, line) in f.lines().enumerate() {
        let line = line.expect("Unable to read line");
        let result = parse_line(&line);
        if let Some(x) = result {
            file.todos.push((cnt, x))
        }
    }

    file
}

fn get_all_todos_from_source_code_files(source_files: &[String]) {
    // TODO (#3): Shall return something
    for source_file in source_files {
        let source_file_todos = get_todos_from_source_code_file(source_file);
        println!(
            "Found the following TODOs for {}:",
            source_file_todos.file_path
        );
        for todo in source_file_todos.todos {
            if todo.1.issue_number > 0 {
                println!(
                    "\tTracked Issue in line {}, number {}: {}",
                    todo.0 + 1,
                    todo.1.issue_number,
                    todo.1.title
                )
            } else {
                println!("\tUntracked issue in line {}: {}", todo.0 + 1, todo.1.title)
            }
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let source_files = get_all_source_code_files()?;
    get_all_todos_from_source_code_files(&source_files);

    // TODO: compare_todos_and_github_issues() has to be implemented
    // TODO: create_new_github_issues() has to be implemented
    // TODO (#123): update_source_code_and_commit() has to be implemented

    Ok(())
}
