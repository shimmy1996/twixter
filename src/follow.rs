use clap::ArgMatches;

use std::fs::OpenOptions;
use std::io::prelude::*;
use std::path::Path;

use crate::config::Config;

/// Follow new source by writing to the config file.
pub fn follow(_config: &Config, subcommand: &ArgMatches, config_path: &Path) {
    let nick = subcommand.value_of("nick").unwrap();
    let url = subcommand.value_of("url").unwrap();
    // Appends given source to end of config file.
    let mut config_file = OpenOptions::new().append(true).open(config_path).unwrap();
    config_file
        .write_fmt(format_args!("\n{} = {}", nick, url))
        .unwrap();
}
