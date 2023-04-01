use git2::Repository;
use log::*;
use std::{error, path::Path, process};

use crate::{
    doit::absolute_path,
    index::{read_index, report_index, Index},
};

// TODO: extract shared code, create better boundary/abstraction
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

    let index = read_index(&repo)?;

    report_index(&index);

    Ok(())
}
