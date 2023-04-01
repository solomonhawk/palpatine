#![allow(unused_variables)]
mod doit;
mod index;

use clap::command;
use clap::{arg, Command};
use std::io;

pub struct Todo {
    author: String,
    body: String,
    severity: usize,
}

fn main() -> Result<(), io::Error> {
    let cmd = command!()
        .propagate_version(true)
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("doit")
                .about(
                    "Catalogs TODOs within a directory and all of its subdirectories (must be within a git repository)",
                )
                .arg(arg!(<PATH>).required(true)),
        );

    let matches = cmd.get_matches();

    match matches.subcommand() {
        Some(("doit", sub_matches)) => {
            doit::run(
                sub_matches
                    .get_one::<String>("PATH")
                    .expect("`doit` requires a <PATH>"),
            )?;
        }
        _ => unreachable!("Exhausted list of subcommands and subcommand_required prevents `None`"),
    }

    Ok(())
}
