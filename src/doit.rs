use std::env;
use std::fs::{self, DirEntry};
use std::io;
use std::path::{Path, PathBuf};

use path_clean::PathClean;

use git2::Repository;

pub fn run(rel_path: &str) -> Result<(), io::Error> {
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

    // recursively walk directories from `path`, collecting all text files
    visit_dirs(&path, &repo, &|entry: &DirEntry| {
        println!("{:?}", entry.path());
    })
}

fn visit_dirs(dir: &Path, repo: &Repository, cb: &dyn Fn(&DirEntry)) -> io::Result<()> {
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
