use std::collections::HashSet;
use std::hash::Hash;
use std::io::{Read, Write};
use std::{env, fs};
use std::{fs::OpenOptions, path::PathBuf};

use mylog::error;
use rayon::iter::{FromParallelIterator, IntoParallelRefIterator, ParallelIterator};
use serde::{Deserialize, Serialize};

const FOLDER_PATH: &str = "sqlformater";
const SETTINGS_PATH: &str = "settings.json";

const DEFAULT_LBA_WORDS: [&str; 4] = [",", "(", "{", ";"];
const DEFAULT_LBA_KEYWORDS: [&str; 6] = ["SELECT", "FROM", "WHERE", "DELETE", "UPDATE", "INSERT"];

const DEFAULT_INDENT_START: [&str; 9] = [
    "(", "[", "{", "SELECT", "FROM", "WHERE", "DELETE", "UPDATE", "INSERT",
];
const DEFAULT_INDENT_END: [&str; 3] = [")", "]", "}"];

const DEFAULT_INDENT_RESET: [&str; 7] =
    [";", "SELECT", "FROM", "WHERE", "DELETE", "UPDATE", "INSERT"];

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    /// Database name (Oracle, PostgreSQL, MySQL, etc)
    pub database: String,
    /// Case of each SQL Keyword
    pub keywords_case: String,
    /// Customize the tabulations
    pub tabulation_format: String,
    /// Insert linebreak after specified words
    /// It could be anything (ponctuation, etc.)
    #[serde(deserialize_with = "deserialize_hashset")]
    pub linebreak_after_words: HashSet<String>,
    /// Insert linebreak after specifieds SQL Keywords
    #[serde(deserialize_with = "deserialize_hashset")]
    pub linebreak_after_keywords: HashSet<String>,
    /// Insert linebreak after specifieds SQL Keywords
    #[serde(deserialize_with = "deserialize_hashset")]
    pub linebreak_before_keywords: HashSet<String>,
    /// Increase indentation after the words
    #[serde(deserialize_with = "deserialize_hashset")]
    pub indentation_start: HashSet<String>,
    /// Decrease indentation after the words
    #[serde(deserialize_with = "deserialize_hashset")]
    pub indentation_end: HashSet<String>,
    /// Reset the indentation adtre the words
    #[serde(deserialize_with = "deserialize_hashset")]
    pub indentation_reset: HashSet<String>,
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
        let linebreak_after_words: HashSet<String> =
            HashSet::from_par_iter(DEFAULT_LBA_WORDS.par_iter().map(|val| val.to_string()));
        let linebreak_after_keywords: HashSet<String> =
            HashSet::from_par_iter(DEFAULT_LBA_KEYWORDS.par_iter().map(|val| val.to_string()));
        let indentation_start: HashSet<String> =
            HashSet::from_par_iter(DEFAULT_INDENT_START.par_iter().map(|val| val.to_string()));
        let indentation_end: HashSet<String> =
            HashSet::from_par_iter(DEFAULT_INDENT_END.par_iter().map(|val| val.to_string()));
        let indentation_reset: HashSet<String> =
            HashSet::from_par_iter(DEFAULT_INDENT_RESET.par_iter().map(|val| val.to_string()));

        Settings {
            database: "generic".to_string(),
            keywords_case: "uppercase".to_string(),
            tabulation_format: "tab1".to_string(),
            linebreak_after_words,
            linebreak_after_keywords: linebreak_after_keywords.clone(),
            linebreak_before_keywords: linebreak_after_keywords,
            indentation_start,
            indentation_end,
            indentation_reset,
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
                            error!(
                                "Errors detected while try to load the settings file : {:?}",
                                path
                            );
                            SavedSettings(Settings::default(), format!("Invalid : {:?}", path))
                        }
                    }
                } else {
                    match SavedSettings::init(Some(path.to_path_buf())) {
                        Ok(savedsettings) => savedsettings,
                        Err(_) => {
                            error!(
                                "Errors detected while try to init the settings folder : {:?}",
                                path
                            );
                            SavedSettings(Settings::default(), format!("Invalid : {:?}", path))
                        }
                    }
                }
            }
            None => match SavedSettings::init(None) {
                Ok(savedsettings) => savedsettings,
                Err(_) => {
                    let current_dir = env::current_dir().unwrap_or_default();
                    error!(
                        "Errors detected while try to init the settings folder : {:?}",
                        current_dir
                    );
                    SavedSettings(Settings::default(), format!("Invalid : {:?}", current_dir))
                }
            },
        }
    }

    fn init(target_path: Option<PathBuf>) -> Result<Self, ()> {
        let path: PathBuf;
        match target_path {
            Some(folder_path) => {
                if !fs::exists(&folder_path).unwrap_or(false) {
                    fs::create_dir_all(&folder_path).map_err(|e| {
                        error!(
                            "Failed to create the directory : {:?}\n\t{}",
                            folder_path, e
                        )
                    })?;
                }
                path = folder_path;
            }
            None => {
                path = env::current_dir()
                    .map_err(|e| error!("{}", e))?
                    .join(FOLDER_PATH);
            }
        }

        let write_files = |input_path: &PathBuf| -> Result<SavedSettings, ()> {
            let setting_path = input_path.join("settings.json");

            if !fs::exists(&setting_path).unwrap_or(false) {
                let mut settings = Settings::default();
                let settings_path =
                    write_settings(input_path, &settings).map_err(|e| error!("{}", e))?;
                settings.tabulation_format = parse_tabulation_format(settings.tabulation_format);
                Ok(SavedSettings(settings, settings_path))
            } else {
                match SavedSettings::from(setting_path) {
                    Ok(savedsettings) => Ok(savedsettings),
                    Err(_) => Err(()),
                }
            }
        };

        if !fs::exists(&path).unwrap_or(false) {
            match fs::create_dir(&path) {
                Ok(_) => write_files(&path),
                Err(error) => {
                    error!("{}", error);
                    Err(())
                }
            }
        } else {
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
            .read(true)
            .open(&settings_path)
            .map_err(|e| error!("{}", e))?;

        let mut content = String::new();
        let _ = file
            .read_to_string(&mut content)
            .map_err(|e| error!("{}", e))?;

        let mut settings: Settings = serde_json::from_str(&content).map_err(|e| error!("{}", e))?;

        settings.tabulation_format = parse_tabulation_format(settings.tabulation_format);

        SavedSettings::update(settings, settings_path)
    }

    fn update(mut settings: Settings, path: PathBuf) -> Result<Self, ()> {
        match settings.keywords_case.as_str() {
            "lowercase" | "lower" => {
                settings.indentation_start = settings
                    .indentation_start
                    .par_iter()
                    .map(|val| val.to_lowercase())
                    .collect();
                settings.indentation_end = settings
                    .indentation_end
                    .par_iter()
                    .map(|val| val.to_lowercase())
                    .collect();
                settings.indentation_reset = settings
                    .indentation_reset
                    .par_iter()
                    .map(|val| val.to_lowercase())
                    .collect();
            }
            "uppercase" | "upper" => {
                settings.indentation_start = settings
                    .indentation_start
                    .par_iter()
                    .map(|val| val.to_uppercase())
                    .collect();
                settings.indentation_end = settings
                    .indentation_end
                    .par_iter()
                    .map(|val| val.to_uppercase())
                    .collect();
                settings.indentation_reset = settings
                    .indentation_reset
                    .par_iter()
                    .map(|val| val.to_uppercase())
                    .collect();
            }
            unsupported_case => {
                error!("Unsupported case : '{}'", unsupported_case);
                return Err(());
            }
        }

        if let Err(error) = write_settings(&mut path.to_path_buf(), &settings) {
            error!("{} - settings path : {}", error, path.display());
            Err(())
        } else {
            Ok(SavedSettings(settings, path.display().to_string()))
        }
    }
}

fn parse_tabulation_format(tabulation_format: String) -> String {
    let result = |pattern: &str, number: String| -> String {
        if let Ok(number) = number.parse::<usize>() {
            pattern.repeat(number)
        } else {
            pattern.to_string()
        }
    };

    if tabulation_format.starts_with("tab") {
        result("\t", tabulation_format.replace("tab", ""))
    } else if tabulation_format.starts_with("space") {
        result(" ", tabulation_format.replace("space", ""))
    } else if tabulation_format == "\t".repeat(tabulation_format.len()) {
        tabulation_format
    } else if tabulation_format == " ".repeat(tabulation_format.len()) {
        tabulation_format
    } else {
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
    } else {
        target_path = path.clone();
    }

    let mut file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(&target_path)
        .map_err(|e| format!("{} when try to create/open the file {:?}", e, path))?;

    let _ = file.write_all(b"*");
    Ok(())
}

fn write_settings(path: &PathBuf, settings: &Settings) -> Result<String, String> {
    let target_path: PathBuf;
    if path.is_dir() {
        target_path = path.join(SETTINGS_PATH);
    } else {
        target_path = path.clone();
    }

    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&target_path)
        .map_err(|e| format!("{}", e))?;

    let json_object = serde_json::to_string(settings).map_err(|e| format!("{}", e))?;

    file.write_all(json_object.as_bytes())
        .map_err(|e| format!("{}", e))?;

    Ok(target_path.display().to_string())
}
