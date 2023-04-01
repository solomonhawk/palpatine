use log::*;
use std::fs::{self, DirEntry};
use std::path::{Path, PathBuf};
use std::{env, error};
use std::{io, process};

use path_clean::PathClean;

use git2::Repository;

use crate::index::{index_file, read_index, report_index, write_index, Index};

pub fn run(rel_path: &str) -> Result<(), Box<dyn error::Error>> {
    let dir_path = absolute_path(&rel_path)
        .map_err(|err| {
            error!("failed to canonicalize the provided path '{rel_path}': {err}");
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

    let mut index: Index = read_index(&repo)?;

    visit_dirs(&path, &repo, &mut |entry: &DirEntry| {
        match index_file(&mut index, &entry, &repo) {
            Err(err) => {
                error!(
                    "failed to index file {path} ({err})",
                    path = entry.path().display()
                );
                process::exit(1);
            }
            _ => (),
        };
    })?;

    report_index(&index);

    write_index(&index, &repo)
}

fn visit_dirs(dir: &Path, repo: &Repository, cb: &mut dyn FnMut(&DirEntry)) -> io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if repo.is_path_ignored(&path).unwrap_or(false) {
                continue;
            }

            if path.is_dir() {
                visit_dirs(&path, &repo, cb)?;
            } else {
                cb(&entry);
            }
        }
    }
    Ok(())
}

fn absolute_path(path: impl AsRef<Path>) -> io::Result<PathBuf> {
    let path = path.as_ref();

    let absolute_path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        env::current_dir()?.join(path)
    }
    .clean();

    Ok(absolute_path)
}
