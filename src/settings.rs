use std::{env, fs};
use std::{fs::OpenOptions, path::PathBuf};
use std::io::{Read, Write};
use std::collections::HashSet;
use std::hash::Hash;

use serde::{Serialize, Deserialize};
use mylog::error;

const FOLDER_PATH: &str = "sqlformater";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    /// Database name (Oracle, PostgreSQL, MySQL, etc)
    pub database: String,
    /// Case of each SQL Keyword
    pub keywords_case: String,
    /// Customize the tabulations
    pub tabulation_format: String,
    /// Insert linebreak after each comma ','
    pub linebreak_after_comma: bool,
    /// Insert linebreak after each left parenthesis '('
    pub linebreak_after_lparenthesis: bool,
    /// Insert linebreak after each left brace '{'
    pub linebreak_after_lbrace: bool,
    /// Insert linebreak after each left bracket '['
    pub linebreak_after_lbracket: bool,
    /// Insert linebreak after each semicolon
    pub linebreak_after_semicolon: bool,
    /// Insert linebreak after specifieds SQL Keywords
    #[serde(deserialize_with = "deserialize_hashset")]
    pub linebreak_after_keywords: HashSet<String>,
    /// Insert linebreak after specifieds SQL Keywords
    #[serde(deserialize_with = "deserialize_hashset")]
    pub linebreak_before_keywords: HashSet<String>,
    /// Insert indentations between the parenthesis
    pub indentation_parenthesis: bool,
    /// Insert indentations between the braces
    pub indentation_braces: bool,
    /// Insert indentations between the brackets
    pub indentation_brackets: bool,
    /// Insert indentations between clauses
    pub indentation_clauses: bool
}

fn deserialize_hashset<'de, D, T>(deserializer: D) -> Result<HashSet<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de> + Eq + Hash,
{
    let vec: Vec<T> = Deserialize::deserialize(deserializer)?;
    Ok(vec.into_iter().collect())
}

impl Default for Settings {
    fn default() -> Settings {
        Settings {
            database: "generic".to_string(),
            keywords_case: "uppercase".to_string(),
            tabulation_format: "\t".to_string(),
            linebreak_after_comma: true,
            linebreak_after_lparenthesis: true,
            linebreak_after_lbrace: true,
            linebreak_after_lbracket: false,
            linebreak_after_semicolon: true,
            linebreak_after_keywords: HashSet::new(),
            linebreak_before_keywords: HashSet::new(),
            indentation_parenthesis: true,
            indentation_braces: true,
            indentation_brackets: false,
            indentation_clauses: false
        }
    }
}

impl Settings {
    pub fn init() -> Result<(), String> {
        let mut path = env::current_dir()
            .map_err(|e| format!("{}", e))?;
        
        path.push(FOLDER_PATH);

        let write_files = |folder_path: &mut PathBuf| -> Result<(), String> {
            let _ = write_gitignore(folder_path)?;
            folder_path.pop();

            if !fs::exists(folder_path.join("settings.json")).unwrap_or(true) {
                write_settings(folder_path, &Settings::default())
            }
            else {
                Ok(())
            }
        };

        if !fs::exists(&path).unwrap_or(false) {
            match fs::create_dir(&path) {
                Ok(_) => {
                    write_files(&mut path)
                },
                Err(error) => Err(format!("{}", error))
            }
        }
        else {
            write_files(&mut path)
        }
    }
    
    pub fn from_file(settings_path: &str) -> Result<Settings, Settings> {
        let mut file = OpenOptions::new()
            .read(true).open(settings_path)
            .map_err(|e| {error!("{}", e); Settings::default()})?;

        let mut content = String::new();
        let _ = file.read_to_string(&mut content)
            .map_err(|e| {error!("{}", e); Settings::default()})?;

        let mut settings: Settings = serde_json::from_str(&content)
            .map_err(|e| {error!("{}", e); Settings::default()})?;

        settings.tabulation_format = parse_tabulation_format(settings.tabulation_format);

        match (settings.indentation_clauses, settings.keywords_case.as_str()) {
            (true, "lowercase" | "lower") => {
                settings.linebreak_after_keywords.insert("select".to_string());
                settings.linebreak_after_keywords.insert("from".to_string());
                settings.linebreak_after_keywords.insert("where".to_string());
                Ok(settings)
            },
            (true, "uppercase" | "upper") => {
                settings.linebreak_after_keywords.insert("SELECT".to_string());
                settings.linebreak_after_keywords.insert("FROM".to_string());
                settings.linebreak_after_keywords.insert("WHERE".to_string());
                Ok(settings)
            },
            (false, "lowercase" | "lower" | "uppercase" | "upper") => {
                Ok(settings)
            }
            (_, unsupported_case) => {
                error!("Unsupported case : '{}'", unsupported_case);
                Err(Settings::default())
            }
        }
    }
}

fn parse_tabulation_format(tabulation_format: String) -> String {
    let result = |pattern: &str, number: String| -> String {
        if let Ok(number) = number.parse::<usize>() {
            pattern.repeat(number)
        }
        else {
            pattern.to_string()
        }
    };

    if tabulation_format.starts_with("tab") {
        result("\t", tabulation_format.replace("tab", ""))
    }
    else if tabulation_format.starts_with("space") {
        result(" ", tabulation_format.replace("space", ""))
    }
    else {
        error!("Unsupported tabulation format : {}", tabulation_format);
        "\t".to_string()
    }
}

fn write_gitignore(folder_path: &mut PathBuf) -> Result<(), String> {
    folder_path.push(".gitignore");

    let mut file = OpenOptions::new()
        .create(true).write(true).open(&folder_path)
        .map_err(|e| format!("{}", e))?;
    
    let _ = file.write_all(b"# Directory of the CLI tool sqlformater\n*\n!.gitignore");
    Ok(())
}

fn write_settings(folder_path: &mut PathBuf, settings: &Settings) -> Result<(), String> {
    folder_path.push("settings.json");

    let mut file = OpenOptions::new()
        .create(true).write(true).truncate(true).open(&folder_path)
        .map_err(|e| format!("{}", e))?;

    let json_object = serde_json::to_string(settings)
        .map_err(|e| format!("{}", e))?;

    let _ = file.write_all(json_object.as_bytes());
    Ok(())
}