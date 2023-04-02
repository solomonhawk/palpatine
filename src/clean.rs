use std::{error, io};

use crate::{config::Config, index::delete_index};

pub fn run(config: &Config) -> Result<(), Box<dyn error::Error>> {
    let index_path = config.index_file_path();

    if !index_path.is_file() {
        println!("No index file to delete");
        return Ok(());
    }

    println!("Are you sure? This will delete the cached index file at .palpatine/index.json [y/n]");

    let input = get_input();

    if input == "y" || input == "Y" {
        delete_index(config)?;
        println!("Clean succeeded");
    } else {
        println!("Clean aborted");
    }

    Ok(())
}

fn get_input() -> String {
    let mut this_input = String::from("");

    io::stdin()
        .read_line(&mut this_input)
        .expect("Failed to read line");

    this_input.trim().to_string()
}
