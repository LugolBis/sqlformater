use std::{collections::HashSet, env, fs::{self, DirEntry}, path::PathBuf};

use mylog::{logs, error};
use rayon::prelude::*;
use crate::settings::{SavedSettings, Settings, write_gitignore};
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
                    Some(p)
                } else {
                    None
                }
            })
        );

        rayon::ThreadPoolBuilder::new()
            .num_threads(
                std::thread::available_parallelism()
                    .map(|n| n.get())
                    .unwrap_or(1)
            )
            .build_global()
            .expect("Error : failed to create the ThreadPool.");

        files_path.par_iter().for_each(|path| {
            let path_string = path.display().to_string();
            match formater(&settings, path.to_path_buf()) {
                Ok(_) => {
                    println!("\nSuccessfully format the file : {}", path_string);
                }
                Err(error) => {
                    error!("{}", error);
                    eprintln!("ERROR : {} with the file : {}", error, path_string);
                }
            }
        });
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
                    logs_path.push_str(&folder_path.display().to_string());
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

fn set_up(settings: &mut Option<Settings>, settings_path: &mut String, logs_path: &mut str) -> Result<(), String> {
    let mut folder_path = env::current_dir().unwrap_or_default();
    folder_path.push("sqlformater");

    if logs_path.is_empty() {
        logs::init(
            folder_path.display().to_string(),
            "1MB".to_string(),
            "7days".to_string()
        )?;
        write_gitignore(&mut folder_path)?;
    }
    else {
        logs::init(
            logs_path.to_owned(),
            "1MB".to_string(),
            "7days".to_string()
        )?;
        let mut path = PathBuf::from(logs_path.to_string());
        write_gitignore(&mut path)?;
    }

    if settings_path.is_empty() {
        let saved_settings = SavedSettings::main(None);
        *settings_path = saved_settings.1;
        *settings = Some(saved_settings.0);
    }
    else {
        let saved_settings = SavedSettings::main(Some(PathBuf::from(settings_path.to_owned())));
        *settings_path = saved_settings.1;
        *settings = Some(saved_settings.0);
    }

    let mut path = PathBuf::from(settings_path.clone());
    write_gitignore(&mut path)?;

    Ok(())
}

fn get_scripts(folder_path: String) -> Vec<PathBuf> {
    let folder_path = PathBuf::from(folder_path);
    let mut scripts_path: Vec<PathBuf> = Vec::new();

    match fs::read_dir(folder_path) {
        Ok(entries) => {
            let entries = entries
                .flatten()
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
