use std::{collections::HashSet, env, fs::{self, DirEntry}, path::PathBuf};

use mylog::{logs, error};
use crate::settings::Settings;
use crate::formater::formater;

const HELP_USAGE: &str = include_str!("../doc/help-usage.txt");
const HELP_SETTINGS: &str = include_str!("../doc/help-settings.txt");

pub fn main(args:Vec<String>) {
    let mut settings_path = String::new();
    let mut settings: Option<Settings> = None;
    let mut logs_path = String::new();
    let mut target_files: HashSet<String> = HashSet::new();
    let mut target_folders: HashSet<String> = HashSet::new();
    let mut help_usage = false;
    let mut help_settings = false;
    let mut verbose = false;
    let mut status = false;

    parse_args(
        args, &mut settings_path, &mut logs_path, &mut target_files,
        &mut target_folders, &mut help_usage, &mut help_settings, &mut verbose, &mut status
    );

    if let Err(error) = set_up(&mut settings, &mut settings_path, &mut logs_path) {
        eprintln!("ERROR : {}", error);
        return;
    }

    if help_usage {
        println!("{}", HELP_USAGE);
    }
    else if help_settings {
        println!("{}", HELP_SETTINGS);
    }
    else if status {
        println!("\nParsed paths :\nLogs path : {}\nSettings path : {}\n",logs_path,settings_path);
    }
    else {
        let settings = settings.unwrap();
        
        let mut files_path: Vec<PathBuf> = Vec::new();
        for folder_path in target_folders {
            files_path.extend(get_scripts(folder_path));
        }

        files_path.extend(
        target_files.iter()
            .filter_map(|s| {
                let p = PathBuf::from(&s);
                if p.exists() {
                    Some(p.to_path_buf())
                } else {
                    None
                }
            })
        );

        for path in files_path {
            let path_string = path.display().to_string();
            match formater(&settings, path) {
                Ok(_) => {
                    println!("\nSuccessfully format the file : {}", path_string);
                }
                Err(error) => {
                    error!("{}", error);
                    eprintln!("ERROR : {} with the file : {}", error, path_string);
                }
            }
        }
    }
}

fn parse_args(
    args:Vec<String>,
    settings_path: &mut String,
    logs_path: &mut String,
    target_files: &mut HashSet<String>,
    target_folders: &mut HashSet<String>,
    help_usage: &mut bool,
    help_settings: &mut bool,
    verbose: &mut bool,
    status: &mut bool
) {
    for arg in args {
        if ["-help", "--help"].contains(&arg.as_str()) {
            *help_usage = true;
        }
        else if ["-help-settings", "--help-settings"].contains(&arg.as_str()) {
            *help_settings = true;
        }
        else if ["-verbose", "--verbose"].contains(&arg.as_str()) {
            *verbose = true;
        }
        else if ["-status", "--status"].contains(&arg.as_str()) {
            *status = true;
        }
        else if [".", "*"].contains(&arg.as_str()) {
            if let Ok(path) = env::current_dir() {
                target_folders.insert(path.display().to_string());
            }
        }
        else if arg.starts_with("-logs_path=") || arg.starts_with("--logs_path=") {
            if let Some(path) = arg.split("=").collect::<Vec<&str>>().get(1) {
                let path = PathBuf::from(path);
                if let Some(folder_path) = path.parent() {
                    logs::init(folder_path.display().to_string());
                    logs_path.push_str(&path.display().to_string());
                }
            }
        }
        else if arg.starts_with("-settings_path=") || arg.starts_with("--settings_path=") {
            if let Some(path) = arg.split("=").collect::<Vec<&str>>().get(1) {
                *settings_path = path.to_string();
            }
        }
        else {
            let path = PathBuf::from(arg);
            if fs::exists(&path).unwrap_or(false) {
                if path.is_dir() {
                    target_folders.insert(path.display().to_string());
                }
                else {
                    let path = path.display().to_string();
                    if path.ends_with(".sql") {
                        target_files.insert(path);
                    }
                }
            }
        }
    }
}

fn set_up(settings: &mut Option<Settings>, settings_path: &mut String, logs_path: &mut String) -> Result<(), String> {
    let mut folder_path = env::current_dir().unwrap_or(PathBuf::new());
    folder_path.push("sqlformater");

    if logs_path.as_str() == "" {
        logs::init(folder_path.display().to_string());
    }
    else {
        logs::init(logs_path.clone());
    }

    if settings_path == "" {
        match Settings::main(None) {
            Ok(loaded_settings) => {
                *settings = Some(loaded_settings);
                Ok(())
            }
            Err(error) => {
                error!("{}", error);
                Err(error)
            }
        }
    }
    else {
        match Settings::main(Some(PathBuf::from(settings_path.clone()))) {
            Ok(loaded_settings) => {
                *settings = Some(loaded_settings);
                Ok(())
            }
            Err(error) => {
                error!("{}", error);
                Err(error)
            }
        }
    }
}

fn get_scripts(folder_path: String) -> Vec<PathBuf> {
    let folder_path = PathBuf::from(folder_path);
    let mut scripts_path: Vec<PathBuf> = Vec::new();

    match fs::read_dir(folder_path) {
        Ok(entries) => {
            let entries = entries
                .filter(|e| e.is_ok())
                .map(|x| x.unwrap())
                .collect::<Vec<DirEntry>>();

            for entry in entries {
                let path = entry.path();
                if path.is_dir() {
                    scripts_path.extend(get_scripts(path.display().to_string()));
                }
                else {
                    let file_name = path.display().to_string();
                    if file_name.ends_with(".sql") {
                        scripts_path.push(path);
                    }
                }
            }

            scripts_path
        },
        Err(error) => {
            error!("{}", error);
            scripts_path
        }
    }
}
