#![allow(unused_variables)]
mod doit;
mod index;
mod report;

use clap::{arg, ArgAction, Command};
use clap::{command, value_parser};

use std::error;
use std::str::FromStr;
use stderrlog::Timestamp;

fn main() -> Result<(), Box<dyn error::Error>> {
    let cmd = command!()
        .arg(arg!(verbosity: -v --verbose)
            .help("Increase message verbosity")
            .action(ArgAction::Count))
        .arg(arg!(-q --quiet)
            .help("Silence all output")
            .action(ArgAction::SetTrue))
        .arg(arg!(-t --timestamp <TS>)
            .help("prepend log lines with a timestamp")
            .value_parser(value_parser!(Timestamp)))
        .propagate_version(true)
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("doit")
                .about(
                    "Catalogs TODOs within a directory and all of its subdirectories (must be within a git repository)",
                )
                .arg(arg!(<PATH>).required(true)),
        )
        .subcommand(Command::new("report").about(
            "Outputs indexed TODOs",
        ));

    let matches = cmd.get_matches();

    let verbose = matches.get_count("verbosity") as usize;
    let quiet = matches.get_flag("quiet");
    let ts = matches
        .get_one::<String>("timestamp")
        .map(|v| {
            stderrlog::Timestamp::from_str(v).unwrap_or_else(|_| {
                clap::Error::raw(
                    clap::error::ErrorKind::InvalidValue,
                    "invalid value for 'timestamp'",
                )
                .exit()
            })
        })
        .unwrap_or(stderrlog::Timestamp::Off);

    stderrlog::new()
        .module(module_path!())
        .quiet(quiet)
        .verbosity(verbose)
        .timestamp(ts)
        .init()
        .unwrap();

    match matches.subcommand() {
        Some(("doit", sub_matches)) => {
            doit::run(
                sub_matches
                    .get_one::<String>("PATH")
                    .expect("`doit` requires a <PATH>"),
            )?;
        }
        Some(("report", _)) => {
            report::run()?;
        }
        _ => unreachable!("Exhausted list of subcommands and subcommand_required prevents `None`"),
    }

    Ok(())
}
