mod config;
mod formater;

use config::Config;
use formater::formater;

fn main() {
    match Config::init() {
        Ok(_) => {
            match Config::from_file("sqlformater/config.json") {
                Ok(config) => {
                    let query = "SeLect AVG(tutu) from toto group by Jojo;";
                    println!("{}", formater(config, query.to_string()).unwrap());
                },
                Err(config) => {
                    println!("Failed to load the config, the default config will be used.");
                    let query = "SeLect AVG(tutu) from toto group by Jojo;";
                    println!("{:?}", formater(config, query.to_string()));
                }
            }
        },
        Err(error) => println!("{}", error)
    }
}
