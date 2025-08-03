use std::{env, fs};
use std::{fs::OpenOptions, path::PathBuf};
use std::io::{Read, Write};

use serde::{Serialize, Deserialize};
use mylog::error;

const FOLDER_PATH: &str = "sqlformater";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Database name (Oracle, PostgreSQL, MySQL, etc)
    pub database: String,
    /// Case of each SQL Keyword
    pub keywords_case: String,
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
    pub linebreak_after_keywords: Vec<String>,
    /// Insert linebreak after specifieds SQL Keywords
    pub linebreak_before_keywords: Vec<String>,
    /// Insert indentations between the parenthesis
    pub indentation_parenthesis: bool,
    /// Insert indentations between the braces
    pub indentation_braces: bool,
    /// Insert indentations between the brackets
    pub indentation_brackets: bool
}

impl Default for Config {
    fn default() -> Config {
        Config {
            database: "generic".to_string(),
            keywords_case: "uppercase".to_string(),
            linebreak_after_comma: true,
            linebreak_after_lparenthesis: true,
            linebreak_after_lbrace: true,
            linebreak_after_lbracket: false,
            linebreak_after_semicolon: true,
            linebreak_after_keywords: vec![],
            linebreak_before_keywords: vec![],
            indentation_parenthesis: true,
            indentation_braces: true,
            indentation_brackets: false
        }
    }
}

impl Config {
    pub fn init() -> Result<(), String> {
        let mut path = env::current_dir()
            .map_err(|e| format!("{}", e))?;
        
        path.push(FOLDER_PATH);

        let write_files = |folder_path: &mut PathBuf| -> Result<(), String> {
            let _ = write_gitignore(folder_path)?;
            folder_path.pop();

            if !fs::exists(folder_path.join("config.json")).unwrap_or(true) {
                write_config(folder_path, &Config::default())
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
    
    pub fn from_file(config_path: &str) -> Result<Config, Config> {
        let mut file = OpenOptions::new()
            .read(true).open(config_path)
            .map_err(|e| {error!("{}", e); Config::default()})?;

        let mut content = String::new();
        let _ = file.read_to_string(&mut content)
            .map_err(|e| {error!("{}", e); Config::default()})?;

        let config: Config = serde_json::from_str(&content)
            .map_err(|e| {error!("{}", e); Config::default()})?;
        
        Ok(config)
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

fn write_config(folder_path: &mut PathBuf, config: &Config) -> Result<(), String> {
    folder_path.push("config.json");

    let mut file = OpenOptions::new()
        .create(true).write(true).truncate(true).open(&folder_path)
        .map_err(|e| format!("{}", e))?;

    let json_object = serde_json::to_string(config)
        .map_err(|e| format!("{}", e))?;

    let _ = file.write_all(json_object.as_bytes());
    Ok(())
}