mod settings;
mod formater;

use std::{env, path::PathBuf};

use settings::Settings;
use formater::formater;
use mylog::logs::init;

fn main() {
    let mut folder_path = env::current_dir().unwrap_or(PathBuf::new());
    folder_path.push("sqlformater");
    init(folder_path.display().to_string());

    match Settings::init() {
        Ok(_) => {
            match Settings::from_file("sqlformater/settings.json") {
                Ok(settings) => {
                    let query = "SeLect Min('date', jojo) from toto where id is not null group by Jojo;";
                    let result = formater(settings, query.to_string()).unwrap();
                    println!("{}\n\n{:?}", result, result);
                },
                Err(settings) => {
                    println!("Failed to load the settings, the default settings will be used.");
                    let query = "SeLect AVG(tutu) from toto group by Jojo;";
                    println!("{:?}", formater(settings, query.to_string()));
                }
            }
        },
        Err(error) => println!("{}", error)
    }
}
