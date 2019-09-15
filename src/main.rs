use clap::{crate_version, App, Arg, ArgMatches, SubCommand};

use std::fs::OpenOptions;
use std::io::prelude::*;
use std::path::Path;
use std::process::Command;

mod config;
use crate::config::Config;

fn main() {
    let command = App::new("twixter")
        .version(crate_version!())
        .about("A client for twtxt, microblog for hackers")
        .arg(
            Arg::with_name("config_dir")
                .short("c")
                .long("config")
                .value_name("PATH")
                .help("Specifies a custom config file location")
                .takes_value(true),
        )
        .subcommand(
            SubCommand::with_name("tweet")
                .version(crate_version!())
                .about("Append a new tweet to your twtxt file")
                .arg(Arg::with_name("content").multiple(true)),
        )
        .subcommand(SubCommand::with_name("timeline").about("Retrieves your timeline"))
        .subcommand(
            SubCommand::with_name("follow")
                .version(crate_version!())
                .about("Adds a new source to your followings")
                .arg(
                    Arg::with_name("nick")
                        .required(true)
                        .value_name("NICK")
                        .help("Specifies nickname to store source with"),
                )
                .arg(
                    Arg::with_name("url")
                        .required(true)
                        .value_name("URL")
                        .help("Specifies source url"),
                ),
        )
        .get_matches();

    // Source config.
    let mut config_dir = command
        .args
        .get("config_dir")
        .and_then(|matched_arg| Some(Path::new(&matched_arg.vals[0]).to_path_buf()))
        .unwrap_or({
            let mut config_dir = dirs::config_dir().unwrap();
            config_dir.push("twixter");
            config_dir
        });
    config_dir.push("config");

    let config = Config::new(&config_dir);

    // Check if twtfile exists and create one if necessary.
    let twtfile_path = Path::new(&config.twtfile).parent().unwrap();
    if !twtfile_path.exists() {
        std::fs::create_dir_all(twtfile_path).unwrap();
    }

    // Parse subcommands.
    match command.subcommand() {
        ("tweet", Some(subcommand)) => tweet(&config, &subcommand),
        _ => {}
    }
}

fn tweet(config: &Config, subcommand: &ArgMatches) {
    // Helper to run the tweet subcommand.

    // Parse tweet content.
    let content = subcommand
        .args
        .get("content")
        .and_then(|matched_arg| {
            Some(
                matched_arg
                    .vals
                    .iter()
                    .map(|os_string| os_string.clone().into_string().unwrap())
                    .collect::<Vec<String>>()
                    .join(" "),
            )
        })
        .unwrap_or_default();

    if content == "" {
        eprintln!("Error: post content must not be empty");
    } else {
        // Run pre tweet hook.
        Command::new("sh")
            .args(&["-c", &config.pre_tweet_hook])
            .output()
            .expect("Failed to run pre tweet hook");

        // Write tweet.
        OpenOptions::new()
            .append(true)
            .create(true)
            .open(Path::new(&config.twtfile))
            .unwrap()
            .write(compose(content).as_bytes())
            .expect("Unable to write new post");

        // Run post tweet hook.
        Command::new("sh")
            .args(&["-c", &config.post_tweet_hook])
            .output()
            .expect("Failed to run post tweet hook");
    }
}

fn compose(content: String) -> String {
    // Formats given content into twtxt format by adding datetime.
    use chrono::{Local, SecondsFormat};
    let timestamp = Local::now().to_rfc3339_opts(SecondsFormat::Secs, true);
    let mut post = String::new();
    post.push_str(&timestamp);
    post.push('\t');
    post.push_str(&content);
    post.push('\n');
    post
}
