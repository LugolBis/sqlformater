use std::{env, fs};

use sqlformater::cli;

const RESULTS_FOLDER: &str = "tests_results";

fn test_cli(args: Vec<&str>, iteration: usize) {
    if (!fs::exists(RESULTS_FOLDER).unwrap_or(false)) && iteration == 0 {
        let _ = fs::create_dir(RESULTS_FOLDER);
    }

    let args = args
        .into_iter()
        .map(|v| v.to_string())
        .collect::<Vec<String>>();

    env::set_current_dir(RESULTS_FOLDER).unwrap_or_else(|_| println!("{}", iteration));
    cli::main(args);
}

#[test]
fn test_initialization() {
    for index in 0..5 {
        test_cli(vec!["*"], index);

        assert!(fs::exists("sqlformater").unwrap_or(false));
        assert!(fs::exists("sqlformater/settings.json").unwrap_or(false));
        assert!(fs::exists("sqlformater/.gitignore").unwrap_or(false));
    }
}

#[test]
fn test_logs_path() {
    for index in 0..5 {
        test_cli(vec!["-logs_path=path_to_logs"], index);

        assert!(fs::exists("path_to_logs").unwrap_or(false));
        assert!(fs::exists("path_to_logs/.gitignore").unwrap_or(false));
    }
}

#[test]
fn test_settings_path() {
    for index in 0..5 {
        test_cli(vec!["-settings_path=path_to_customized_settings"], index);

        assert!(fs::exists("path_to_customized_settings").unwrap_or(false));
        assert!(fs::exists("path_to_customized_settings/settings.json").unwrap_or(false));
        assert!(fs::exists("path_to_customized_settings/.gitignore").unwrap_or(false));
    }
}
