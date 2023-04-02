use std::error;

use crate::config::Config;
use crate::index::{read_index, report_index};

pub fn run(config: &Config) -> Result<(), Box<dyn error::Error>> {
    let index = read_index(config)?;

    report_index(&index);

    Ok(())
}
