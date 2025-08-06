const USAGE: &str = r#"
Usage:
    sqlformater <PATHS> [OPTIONS]

<PATHS>:
    Specify which SQL scripts to format. You can provide one or more of the following :

    - Folder paths : applies formatting to all .sql files in the subdirectories.
    - File paths : must be a file with the .sql extension.
    - '.' or '*' : selects the current directory and all its subdirectories.

<OPTIONS>:
    -logs_path=<FOLDER_PATH>, --logs_path=<FOLDER_PATH> :
        Set the path to the logs directory.

    -settings_path=<FILE_PATH>, --settings_path=<FILE_PATH> :
        Set the path to the configuration file to load.

    -verbose, --verbose :
        Enable verbose mode to display detailed execution information.

    -help, --help :
        Display this help message.

    -status, --status :
        Show information about settings, logs, and other details.
"#;

pub fn print_help() {
    println!("{}", USAGE);
}