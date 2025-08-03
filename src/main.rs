mod config;
mod formater;

use config::Config;
use formater::formater;

fn main() {
    match Config::init() {
        Ok(_) => {
            match Config::from_file("sqlformater/config.json") {
                Ok(config) => {
                    let query = "SeLect Min('tutu', toto, jojo) from toto where id is not null group by Jojo;";
                    let result = formater(config, query.to_string()).unwrap();
                    println!("{}\n\n{:?}", result, result);
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
