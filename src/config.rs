use crate::iniparser;

use std::env;

#[derive(Debug, Clone)]
pub struct GitHubConfig {
    pub token: String,
}

fn get_github_token_from_env() -> Option<String> {
    if env::var("GITHUB_TOKEN").is_err() {
        return None;
    }
    Some(env::var("GITHUB_TOKEN").unwrap())
}

impl GitHubConfig {
    pub fn new(token: &str) -> GitHubConfig {
        GitHubConfig {
            token: token.to_string(),
        }
    }
    pub fn from_env() -> Option<GitHubConfig> {
        if let Some(github_token) = get_github_token_from_env() {
            Some(GitHubConfig::new(&github_token))
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub struct GitLabConfig {
    pub host: String,
    pub token: String,
}

fn get_gitlab_token_from_env() -> Option<String> {
    if env::var("GITLAB_TOKEN").is_err() {
        return None;
    }
    Some(env::var("GITLAB_TOKEN").unwrap())
}

impl GitLabConfig {
    pub fn new(host: &str, token: &str) -> GitLabConfig {
        GitLabConfig {
            host: host.to_string(),
            token: token.to_string(),
        }
    }
    pub fn from_env() -> Vec<GitLabConfig> {
        let mut gitlab_configs: Vec<GitLabConfig> = Vec::new();
        if let Some(gitlab_token_env) = get_gitlab_token_from_env() {
            let gitlab_tokens: Vec<&str> = gitlab_token_env.split(';').collect();
            for token in gitlab_tokens {
                let token_host: Vec<&str> = token.split(':').collect();
                if token_host.len() == 1 {
                    gitlab_configs.push(GitLabConfig::new("", token_host.get(0).unwrap()))
                } else if token_host.len() == 2 {
                    gitlab_configs.push(GitLabConfig::new(
                        token_host.get(0).unwrap(),
                        token_host.get(1).unwrap(),
                    ))
                } else {
                    eprintln!("Error parsing GITLAB_TOKEN. Read README.md and check it");
                    std::process::exit(1);
                }
            }
        }
        gitlab_configs
    }
}

#[derive(Debug)]
pub struct GeneralConfig {
    pub ignored_extensions: Vec<String>,
}

impl GeneralConfig {
    fn new() -> GeneralConfig {
        GeneralConfig {
            ignored_extensions: Vec::<String>::new(),
        }
    }
}

#[derive(Debug)]
pub struct Config {
    pub general: GeneralConfig,
    pub github: Option<GitHubConfig>,
    pub gitlab: Vec<GitLabConfig>,
}

impl Config {
    pub fn from_file(file_name: &std::path::PathBuf) -> Result<Config, String> {
        match iniparser::parse_ini_file(&file_name.to_str().unwrap()) {
            Ok(ini) => {
                let mut config = Config {
                    general: GeneralConfig::new(),
                    github: None,
                    gitlab: Vec::<GitLabConfig>::new(),
                };
                for section in ini.sections() {
                    let section_name = section.name();
                    if !section_name.is_empty() {
                        if section_name == "general" {
                            if let Ok(ignored_extensions) = section.get("ignored_extensions") {
                                let ignored_extensions: Vec<String> = ignored_extensions
                                    .split(';')
                                    .map(|x| x.to_string())
                                    .collect();
                                config.general = GeneralConfig { ignored_extensions };
                            }
                            continue;
                        }
                        if let Ok(token) = section.get("token") {
                            if section_name == "github.com" {
                                config.github = Some(GitHubConfig::new(token));
                            } else {
                                config.gitlab.push(GitLabConfig::new(section_name, token));
                            }
                        } else {
                            println!(
                                "Warning: No token found in section {}. Skipping",
                                section_name
                            );
                        }
                    } else {
                        println!("Warning: Encountered empty section name in config. Skipping");
                    }
                }
                Ok(config)
            }
            Err(e) => Err(e),
        }
    }
    pub fn from_env() -> Result<Config, String> {
        let config = Config {
            general: GeneralConfig::new(),
            github: GitHubConfig::from_env(),
            gitlab: GitLabConfig::from_env(),
        };
        if config.github.is_none() && config.github.is_none() {
            Err("Could not construct any config from env variables".to_string())
        } else {
            Ok(config)
        }
    }
}
