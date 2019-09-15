use clap::ArgMatches;

use std::fs::OpenOptions;
use std::io::prelude::*;
use std::path::Path;
use std::process::Command;

use crate::config::Config;

/// Helper to run the tweet subcommand.
pub fn tweet(config: &Config, subcommand: &ArgMatches) {
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

/// Formats given content into twtxt format by adding datetime.
fn compose(content: String) -> String {
    use chrono::{Local, SecondsFormat};
    let timestamp = Local::now().to_rfc3339_opts(SecondsFormat::Secs, true);
    let mut post = String::new();
    post.push_str(&timestamp);
    post.push('\t');
    post.push_str(&content);
    post.push('\n');
    post
}
