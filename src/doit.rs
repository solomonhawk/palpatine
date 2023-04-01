use std::collections::HashMap;
use std::fs::{self, DirEntry};
use std::io;
use std::path::{Path, PathBuf};
use std::{env, error};

use path_clean::PathClean;

use git2::Repository;

use crate::index::{self, index_file, write_index, Index};

pub fn run(rel_path: &str) -> Result<(), Box<dyn error::Error>> {
    let dir_path = absolute_path(&rel_path)
        .map_err(|err| {
            eprintln!("ERROR: failed to canonicalize the provided path '{rel_path}': {err}");
        })
        .unwrap();

    let path = Path::new(&dir_path);

    if !path.is_dir() {
        eprintln!("ERROR: the PATH {path:?} is not a directory");
    }

    let repo = Repository::discover(path).map_err(|err| {
        eprintln!(
            "ERROR: the provided path {path:?} does not appear to exist within a git repository ({err})"
        )
    }).unwrap();

    let mut index: Index = HashMap::new();

    // recursively walk directories from `path`, collecting all text files
    visit_dirs(&path, &repo, &mut |entry: &DirEntry| {
        println!("{:?}", entry.path());

        // TODO: load index and pass in previous indexed_entry
        match index_file(&entry, &repo) {
            Err(err) => {
                eprintln!("{err:?}");
            }
            Ok(indexed_entry) => {
                index.insert(indexed_entry.path.clone(), indexed_entry);
            }
        }
    })?;

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
