use clap::ArgMatches;
use reqwest::Client;

use std::collections::BinaryHeap;

use crate::config::Config;
use crate::entry::Entry;

/// Print timeline from following.
pub fn timeline(config: &Config, _subcommand: &ArgMatches) {
    // Store (post_time, nick, content).
    let mut all_tweets = BinaryHeap::<Entry>::new();

    // Pull and parse twtxt files from user and each followed source.
    for (nick, twturl) in config
        .following
        .iter()
        .chain(vec![(&config.nick, &config.twturl)].into_iter())
    {
        all_tweets.append(&mut parse_twtxt(twturl, nick));
    }

    // Print the most recent tweets.
    for _ in 0..config.limit_timeline {
        if let Some(tweet) = all_tweets.pop() {
            if config.use_abs_time {
                println!("{:#}", tweet);
            } else {
                println!("{}", tweet);
            }
        }
    }
}

/// Parses given twtxt url, returns a Vec of (post_time, content).
fn parse_twtxt(twturl: &str, nick: &str) -> BinaryHeap<Entry> {
    let client = Client::new();
    let mut tweets = BinaryHeap::new();

    if let Ok(resp_text) = client.get(twturl).send().and_then(|mut resp| resp.text()) {
        for line in resp_text.lines() {
            if let Ok(parsed) = Entry::parse(line, Some(nick)) {
                tweets.push(parsed);
            };
        }
    };

    tweets
}
