use std::{env, fs};
use std::{fs::OpenOptions, path::PathBuf};
use std::io::{Read, Write};
use std::collections::HashSet;
use std::hash::Hash;

use serde::{Serialize, Deserialize};
use mylog::error;

const FOLDER_PATH: &str = "sqlformater";
const SETTINGS_PATH: &str = "settings.json";

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

pub struct SavedSettings(pub Settings, pub String);

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
        let mut hash_after: HashSet<String> = HashSet::new();
        hash_after.insert("SELECT".to_string());
        hash_after.insert("FROM".to_string());
        hash_after.insert("WHERE".to_string());

        Settings {
            database: "generic".to_string(),
            keywords_case: "uppercase".to_string(),
            tabulation_format: "tab1".to_string(),
            linebreak_after_comma: true,
            linebreak_after_lparenthesis: true,
            linebreak_after_lbrace: true,
            linebreak_after_lbracket: false,
            linebreak_after_semicolon: true,
            linebreak_after_keywords: hash_after,
            linebreak_before_keywords: HashSet::new(),
            indentation_parenthesis: true,
            indentation_braces: true,
            indentation_brackets: false,
            indentation_clauses: true
        }
    }
}

impl SavedSettings {
    pub fn main(path: Option<PathBuf>) -> Self {
        match &path {
            Some(path) => {
                if path.is_file() {
                    match SavedSettings::from(path.to_path_buf()) {
                        Ok(savedsettings) => savedsettings,
                        Err(_) => {
                            error!("Errors detected while try to load the settings file : {:?}", path);
                            SavedSettings(Settings::default(), format!("Invalid : {:?}", path))
                        }
                    }
                }
                else {
                    match SavedSettings::init(Some(path.to_path_buf())) {
                        Ok(savedsettings) => savedsettings,
                        Err(_) => {
                            error!("Errors detected while try to init the settings folder : {:?}", path);
                            SavedSettings(Settings::default(), format!("Invalid : {:?}", path))
                        }
                    }
                }
            },
            None => {
                match SavedSettings::init(None) {
                    Ok(savedsettings) => savedsettings,
                    Err(_) => {
                        let current_dir = env::current_dir()
                            .unwrap_or_default();
                        error!("Errors detected while try to init the settings folder : {:?}", current_dir);
                        SavedSettings(Settings::default(), format!("Invalid : {:?}", current_dir))
                    }
                }
            }
        }
    }

    fn init(target_path: Option<PathBuf>) -> Result<Self, ()> {
        let path: PathBuf;
        match target_path {
            Some(folder_path) => {
                if !fs::exists(&folder_path).unwrap_or(false) {
                    fs::create_dir_all(&folder_path)
                        .map_err(|e| error!(
                            "Failed to create the directory : {:?}\n\t{}", folder_path, e
                        ))?;
                }
                path = folder_path;
            },
            None => {
                path = env::current_dir()
                    .map_err(|e| error!("{}", e))?
                    .join(FOLDER_PATH);
            }
        }

        let write_files = |input_path: &PathBuf| -> Result<SavedSettings, ()>
        {
            let setting_path = input_path.join("settings.json");

            if !fs::exists(&setting_path).unwrap_or(false) {
                let mut settings = Settings::default();
                let settings_path = write_settings(input_path, &settings)
                    .map_err(|e| error!("{}", e))?;
                settings.tabulation_format = parse_tabulation_format(settings.tabulation_format);
                Ok(SavedSettings(settings, settings_path))
            }
            else {
                match SavedSettings::from(setting_path) {
                    Ok(savedsettings) => Ok(savedsettings),
                    Err(_) => {
                        Err(())
                    }
                }
            }
        };

        if !fs::exists(&path).unwrap_or(false) {
            match fs::create_dir(&path) {
                Ok(_) => {
                    write_files(&path)
                },
                Err(error) => {
                    error!("{}", error);
                    Err(())
                }
            }
        }
        else {
            write_files(&path)
        }
    }
    
    /// This function extract the settings from the path in input, it could be the path of the 'settings.json'
    /// or a folder who's contains a file 'settings.json'
    fn from(path: PathBuf) -> Result<Self, ()> {
        let mut settings_path = path;
        if settings_path.is_dir() {
            settings_path.push(SETTINGS_PATH)
        }

        let mut file = OpenOptions::new()
            .read(true).open(&settings_path)
            .map_err(|e| error!("{}", e))?;

        let mut content = String::new();
        let _ = file.read_to_string(&mut content)
            .map_err(|e| error!("{}", e))?;

        let mut settings: Settings = serde_json::from_str(&content)
            .map_err(|e| error!("{}", e))?;

        settings.tabulation_format = parse_tabulation_format(settings.tabulation_format);

        SavedSettings::update(settings, settings_path)
    }

    fn update(mut settings: Settings, path: PathBuf) -> Result<Self, ()> {
        match (settings.indentation_clauses, settings.keywords_case.as_str()) {
            (true, "lowercase" | "lower") => {
                if settings.linebreak_after_keywords.insert("select".to_string())
                    || settings.linebreak_after_keywords.insert("from".to_string())
                    || settings.linebreak_after_keywords.insert("where".to_string())
                {
                    if let Err(error) = write_settings(&mut path.to_path_buf(), &settings) {
                        error!("{} - settings path : {}", error, path.display());
                        return Err(());
                    }
                }
                Ok(SavedSettings(settings, path.display().to_string()))
            },
            (true, "uppercase" | "upper") => {
                if settings.linebreak_after_keywords.insert("SELECT".to_string())
                    || settings.linebreak_after_keywords.insert("FROM".to_string())
                    || settings.linebreak_after_keywords.insert("WHERE".to_string())
                {
                    if let Err(error) = write_settings(&mut path.to_path_buf(), &settings) {
                        error!("{} - settings path : {}", error, path.display());
                        return Err(());
                    }
                }
                Ok(SavedSettings(settings, path.display().to_string()))
            },
            (false, "lowercase" | "lower" | "uppercase" | "upper") => {
                Ok(SavedSettings(settings, path.display().to_string()))
            }
            (_, unsupported_case) => {
                error!("Unsupported case : '{}'", unsupported_case);
                Err(())
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

pub fn write_gitignore(path: &PathBuf) -> Result<(), String> {
    let target_path: PathBuf;
    if path.is_dir() {
        if !fs::exists(path).unwrap_or(false) {
            fs::create_dir_all(path).map_err(|e| format!("{}", e))?;
        }
        target_path = path.join(".gitignore");
    }
    else {
        target_path = path.clone();
    }

    let mut file = OpenOptions::new()
        .create(true).truncate(true).write(true).open(&target_path)
        .map_err(|e| format!("{} when try to create/open the file {:?}", e, path))?;
    
    let _ = file.write_all(b"*");
    Ok(())
}

fn write_settings(path: &PathBuf, settings: &Settings) -> Result<String, String> {
    let target_path: PathBuf;
    if path.is_dir() {
        target_path = path.join(SETTINGS_PATH);
    }
    else {
        target_path = path.clone();
    }

    let mut file = OpenOptions::new()
        .create(true).write(true).truncate(true).open(&target_path)
        .map_err(|e| format!("{}", e))?;

    let json_object = serde_json::to_string(settings)
        .map_err(|e| format!("{}", e))?;

    file.write_all(json_object.as_bytes())
        .map_err(|e| format!("{}", e))?;

    Ok(target_path.display().to_string())
}
