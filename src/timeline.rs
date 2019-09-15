use chrono::{DateTime, FixedOffset, Local, SecondsFormat};
use clap::ArgMatches;
use reqwest::Client;

use std::collections::BinaryHeap;

use crate::config::Config;

/// Print timeline from following.
pub fn timeline(config: &Config, _subcommand: &ArgMatches) {
    // Store (post_time, nick, content).
    let mut all_tweets = BinaryHeap::<(DateTime<FixedOffset>, String, String)>::new();

    for (nick, twturl) in config.following.iter() {
        let tweets = parse_twtxt(twturl);
        for (post_time, content) in tweets {
            all_tweets.push((post_time, nick.to_owned(), content));
        }
    }

    // Print the most recent tweets.
    let now = Local::now();
    for _ in 0..config.limit_timeline {
        if let Some(tweet) = all_tweets.pop() {
            println!("{}", format_tweet(&tweet, &now));
        }
    }
}

/// Parses given twtxt url, returns a Vec of (post_time, content).
fn parse_twtxt(twturl: &str) -> Vec<(DateTime<FixedOffset>, String)> {
    let client = Client::new();
    let resp = client.get(twturl).send().unwrap().text().unwrap();
    let mut tweets = Vec::new();

    for line in resp.lines() {
        if let Some(seperator_idx) = line.find('\t') {
            if let Ok(tweet_time) = DateTime::parse_from_rfc3339(&line[..seperator_idx]) {
                tweets.push((tweet_time, line[seperator_idx + 1..].to_owned()));
            };
        };
    }

    tweets
}

/// Formats a tweet for display in terminal.
fn format_tweet(tweet: &(DateTime<FixedOffset>, String, String), now: &DateTime<Local>) -> String {
    format!(
        "\n@{} {}\n{}",
        &tweet.1,
        &tweet
            .0
            .with_timezone(&now.timezone())
            .to_rfc3339_opts(SecondsFormat::Secs, true),
        &tweet.2
    )
}
