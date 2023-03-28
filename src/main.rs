#![allow(unused_variables, unused_imports)]
use clap::command;
use clap::{arg, Command};
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use path_clean::PathClean;

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

fn doit(rel_path: &str) {
    let dir_path = absolute_path(&rel_path)
        .map_err(|err| {
            eprintln!("ERROR: failed to canonicalize the provided path '{rel_path}': {err}");
        })
        .unwrap();

    let path = Path::new(&dir_path);

    if !path.is_dir() {
        eprintln!("ERROR: the PATH {path:?} is not a directory");
    }
}

fn main() {
    let cmd = command!()
        .propagate_version(true)
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("doit")
                .about("Catalogues TODOs within a directory and all of its subdirectories")
                .arg(arg!(<PATH>).required(true)),
        );

    let matches = cmd.get_matches();

    match matches.subcommand() {
        Some(("doit", sub_matches)) => doit(
            sub_matches
                .get_one::<String>("PATH")
                .expect("`doit` requires a <PATH>"),
        ),
        _ => unreachable!("Exhausted list of subcommands and subcommand_required prevents `None`"),
    }
}
