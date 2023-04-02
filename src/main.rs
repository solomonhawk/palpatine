mod clean;
mod config;
mod doit;
mod index;
mod report;

use clap::{arg, ArgAction, Command};
use clap::{command, value_parser};
use config::Config;

use std::error;
use std::path::Path;
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
                .arg(arg!(<PATH>).default_missing_value(".")),
        )
        .subcommand(
            Command::new("report")
                .about("Outputs indexed TODOs")
                .arg(arg!(<PATH>).default_missing_value("."))
        )
        .subcommand(
            Command::new("clean")
                .about("Removes the cached TODOs index")
                .arg(arg!(<PATH>).default_missing_value("."))
        );

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
            let path = Path::new(
                sub_matches
                    .get_one::<String>("PATH")
                    .expect("`doit` requires a <PATH>"),
            );
            doit::run(&Config::new(Some(path.into())))?;
        }
        Some(("report", sub_matches)) => {
            let path = Path::new(
                sub_matches
                    .get_one::<String>("PATH")
                    .expect("`report` requires a <PATH>"),
            );
            report::run(&Config::new(Some(path.into())))?;
        }
        Some(("clean", sub_matches)) => {
            let path = Path::new(
                sub_matches
                    .get_one::<String>("PATH")
                    .expect("`clean` requires a <PATH>"),
            );
            clean::run(&Config::new(Some(path.into())))?;
        }
        _ => unreachable!("Exhausted list of subcommands and subcommand_required prevents `None`"),
    }

    Ok(())
}
