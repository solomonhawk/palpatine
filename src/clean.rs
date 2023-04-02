use git2::Repository;
use log::*;
use std::{error, io, path::Path, process};

use crate::{doit::absolute_path, index::delete_index};

pub fn run() -> Result<(), Box<dyn error::Error>> {
    let dir_path = absolute_path(Path::new("."))
        .map_err(|err| {
            // TODO: this error doesn't make a ton of sense
            error!("failed to canonicalize the provided path '.': {err}");
            process::exit(1);
        })
        .unwrap();

    let path = Path::new(&dir_path);

    if !path.is_dir() {
        error!("the PATH {path:?} is not a directory");
        process::exit(1);
    }

    let repo = Repository::discover(path)
        .map_err(|err| {
            error!(
           "the provided path {path:?} does not appear to exist within a git repository ({err})"
          );
            process::exit(1);
        })
        .unwrap();

    let workdir = repo
        .workdir()
        .expect("ERROR: could not find workdir, is this a git directory or subdirectory?");

    let index_path = Path::new(workdir).join(".palpatine/index.json");

    if !index_path.is_file() {
        println!("No index file to delete");
        return Ok(());
    }

    println!("Are you sure? This will delete the cached index file at .palpatine/index.json [y/n]");

    let input = get_input();

    if input == "y" || input == "Y" {
        delete_index(&repo)?;
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
