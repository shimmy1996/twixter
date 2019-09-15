use chrono::{DateTime, Duration, FixedOffset, Local, SecondsFormat};
use clap::ArgMatches;
use reqwest::Client;

use std::collections::BinaryHeap;

use crate::config::Config;

/// Print timeline from following.
pub fn timeline(config: &Config, _subcommand: &ArgMatches) {
    // Store (post_time, nick, content).
    let mut all_tweets = BinaryHeap::<(DateTime<FixedOffset>, String, String)>::new();

    // Pull and parse twtxt files from user and each followed source.
    for (nick, twturl) in config
        .following
        .iter()
        .chain(vec![(&config.nick, &config.twturl)].into_iter())
    {
        let tweets = parse_twtxt(twturl);
        for (post_time, content) in tweets {
            all_tweets.push((post_time, nick.to_owned(), content));
        }
    }

    // Print the most recent tweets.
    let now = Local::now();
    for _ in 0..config.limit_timeline {
        if let Some(tweet) = all_tweets.pop() {
            println!("{}", format_tweet(&tweet, &now, config.use_abs_time));
        }
    }
}

/// Parses given twtxt url, returns a Vec of (post_time, content).
fn parse_twtxt(twturl: &str) -> Vec<(DateTime<FixedOffset>, String)> {
    let client = Client::new();
    let mut tweets = Vec::new();

    if let Ok(resp_text) = client.get(twturl).send().and_then(|mut resp| resp.text()) {
        for line in resp_text.lines() {
            if let Some(seperator_idx) = line.find('\t') {
                if let Ok(tweet_time) = DateTime::parse_from_rfc3339(&line[..seperator_idx]) {
                    tweets.push((tweet_time, line[seperator_idx + 1..].to_owned()));
                };
            };
        }
    };

    tweets
}

/// Formats a tweet for display in terminal.
fn format_tweet(
    tweet: &(DateTime<FixedOffset>, String, String),
    now: &DateTime<Local>,
    use_abs_time: bool,
) -> String {
    let timestamp = if use_abs_time {
        tweet
            .0
            .with_timezone(&now.timezone())
            .to_rfc3339_opts(SecondsFormat::Secs, true)
    } else {
        format_duration(*now - tweet.0.with_timezone(&now.timezone()))
    };

    format!("\n@{} {}\n{}", &tweet.1, &timestamp, &tweet.2)
}

/// Formats a time duration in human readable format.
/// Shows the first non-zero amount of year, month, week, day, hour, or
/// minute. For duration shorter than one minute, return "just now".
fn format_duration(duration: Duration) -> String {
    let (num, unit) = if duration.num_days() >= 365 {
        (duration.num_days() / 365, "year")
    } else if duration.num_days() >= 30 {
        (duration.num_days() / 30, "month")
    } else if duration.num_weeks() >= 1 {
        (duration.num_weeks(), "week")
    } else if duration.num_days() >= 1 {
        (duration.num_days(), "day")
    } else if duration.num_hours() >= 1 {
        (duration.num_hours(), "hour")
    } else if duration.num_minutes() >= 1 {
        (duration.num_minutes(), "minute")
    } else {
        return "just now".to_string();
    };

    if num > 1 {
        format!("{} {}s ago", num, unit)
    } else {
        format!("{} {} ago", num, unit)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(Duration::days(365 * 2)), "2 years ago");
        assert_eq!(format_duration(Duration::days(365)), "1 year ago");
        assert_eq!(format_duration(Duration::days(30 * 3)), "3 months ago");
        assert_eq!(format_duration(Duration::days(30)), "1 month ago");
        assert_eq!(format_duration(Duration::weeks(4)), "4 weeks ago");
        assert_eq!(format_duration(Duration::weeks(1)), "1 week ago");
        assert_eq!(format_duration(Duration::days(4)), "4 days ago");
        assert_eq!(format_duration(Duration::days(1)), "1 day ago");
        assert_eq!(format_duration(Duration::hours(23)), "23 hours ago");
        assert_eq!(format_duration(Duration::hours(1)), "1 hour ago");
        assert_eq!(format_duration(Duration::minutes(5)), "5 minutes ago");
        assert_eq!(format_duration(Duration::minutes(1)), "1 minute ago");
        assert_eq!(format_duration(Duration::seconds(30)), "just now");
    }
}
