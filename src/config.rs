use std::{
    env, io,
    path::{Path, PathBuf},
    process,
};

use git2::Repository;
use log::*;

use path_clean::PathClean;

pub struct Config {
    // where to start searching for TODOs
    pub path: Option<Box<Path>>,

    // the associated git repo discovered based on the path
    pub repo: Repository,
}

impl Config {
    pub fn new(rel_path: Option<Box<Path>>) -> Self {
        let rel_path = rel_path.unwrap_or(Path::new(".").into());
        let root_path = absolute_path(&rel_path)
            .map_err(|err| {
                error!("failed to canonicalize the provided path '{rel_path:?}': {err}");
                process::exit(1);
            })
            .unwrap();

        if !root_path.is_dir() {
            error!("the PATH {root_path:?} is not a directory");
            process::exit(1);
        }

        let repo = Repository::discover(&root_path)
            .map_err(|err| {
                error!(
              "the provided path {root_path:?} does not appear to exist within a git repository ({err})"
              );
                process::exit(1);
            })
            .unwrap();

        Config {
            path: Some(rel_path),
            repo,
        }
    }

    pub fn root_dir(&self) -> PathBuf {
        self.repo
            .workdir()
            .expect("ERROR: could not find workdir, is this a git directory or subdirectory?")
            .into()
    }

    pub fn index_dir(&self) -> PathBuf {
        Path::new(&self.root_dir()).join(".palpatine")
    }

    pub fn index_file_path(&self) -> PathBuf {
        Path::new(&self.root_dir()).join(".palpatine/index.json")
    }
}

pub fn absolute_path(path: impl AsRef<Path>) -> io::Result<PathBuf> {
    let path = path.as_ref();

    let absolute_path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        env::current_dir()?.join(path)
    }
    .clean();

    Ok(absolute_path)
}
