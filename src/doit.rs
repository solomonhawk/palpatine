use log::*;
use std::error;
use std::fs::{self, DirEntry};
use std::path::Path;
use std::{io, process};

use crate::config::Config;
use crate::index::{index_file, read_index, report_index, write_index};

pub fn run(config: &Config) -> Result<(), Box<dyn error::Error>> {
    let mut index = read_index(config)?;
    let mut indexed_count = 0;

    // TODO: can this be multi-threaded?
    visit_dirs(&config.root_dir(), config, &mut |entry: &DirEntry| {
        match index_file(&mut index, &entry, config, &mut indexed_count) {
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

    println!("{indexed_count} file(s) were updated");

    report_index(&index);
    write_index(&index, config)
}

fn visit_dirs(dir: &Path, config: &Config, cb: &mut dyn FnMut(&DirEntry)) -> io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if config.repo.is_path_ignored(&path).unwrap_or(false) {
                continue;
            }

            if path.is_dir() {
                visit_dirs(&path, config, cb)?;
            } else {
                cb(&entry);
            }
        }
    }
    Ok(())
}
