use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug, Clone)]
pub struct Key {
    name: String,
    value: String,
}

impl Key {
    fn value(&self) -> &str {
        &self.value
    }
}

pub struct Section {
    name: String,
    keys: Vec<Key>,
}

impl Section {
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn get(&self, name: &str) -> Result<&str, String> {
        for key in &self.keys {
            if key.name == name {
                return Ok(&key.value());
            }
        }
        Err("Not found".to_string())
    }
}

pub struct Ini {
    sections: Vec<Section>,
}

impl Ini {
    fn new() -> Ini {
        Ini {
            sections: Vec::<Section>::new(),
        }
    }
    pub fn section(&self, name: &str) -> Result<&Section, String> {
        for section in &self.sections {
            if section.name == name {
                return Ok(&section);
            }
        }
        Err("Not found".to_string())
    }
    pub fn sections(&self) -> &Vec<Section> {
        &self.sections
    }
}

pub fn parse_ini_file(file_name: &str) -> Result<Ini, String> {
    // TODO (#15): Make ini parsing prettier
    let f = File::open(file_name).expect("Unable to open file");
    let f = BufReader::new(f);

    let mut ini: Ini = Ini::new();

    let mut section_name: String = "".to_string();
    let mut keys: Vec<Key> = Vec::new();
    for line in f.lines() {
        let line = line.expect("Unable to read line");
        let trimmed_line = line.trim();
        if trimmed_line.is_empty() {
            continue;
        }
        let patterns: &[_] = &['#', ';'];
        if trimmed_line.starts_with(patterns) {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            if !keys.is_empty() {
                let section: Section = Section {
                    name: section_name,
                    keys: keys.clone(),
                };
                ini.sections.push(section);
                keys.clear();
            }
            let patterns: &[_] = &['[', ']'];
            section_name = trimmed_line.trim_matches(patterns).trim().to_string();
        } else {
            let v: Vec<&str> = trimmed_line.split('=').collect();
            if v.len() < 2 {
                println!("Error parsing line '{}'. Skipping it.", trimmed_line);
                continue;
            }
            keys.push(Key {
                name: (*v.get(0).unwrap()).trim().to_string(),
                value: (*v.get(1).unwrap()).trim().to_string(),
            });
        }
    }

    if !keys.is_empty() {
        let section: Section = Section {
            name: section_name,
            keys,
        };
        ini.sections.push(section);
    }

    Ok(ini)
}
