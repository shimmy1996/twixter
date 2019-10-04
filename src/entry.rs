use chrono::{DateTime, Duration, FixedOffset, Local, SecondsFormat};

use std::fmt;

#[derive(Ord, PartialOrd, PartialEq, Eq)]
pub struct Entry {
    pub timestamp: DateTime<FixedOffset>,
    pub author: Option<String>,
    pub content: String,
}

impl Entry {
    /// Generates a new entry from given content.
    pub fn new(content: String, author: Option<String>) -> Entry {
        Entry {
            timestamp: Local::now().into(),
            author,
            content,
        }
    }

    /// Attempt to parse given line into entry format.
    pub fn parse(line: &str, author: Option<&str>) -> Result<Entry, ()> {
        if let Some(seperator_idx) = line.find('\t') {
            if let Ok(timestamp) = DateTime::parse_from_rfc3339(&line[..seperator_idx]) {
                Ok(Entry {
                    timestamp,
                    author: author.and_then(|x| Some(x.to_owned())),
                    content: line[seperator_idx + 1..].to_owned(),
                })
            } else {
                Err(())
            }
        } else {
            Err(())
        }
    }

    /// Generates string representation per twtxt spec for given entry.
    pub fn to_twtxt(&self) -> String {
        format!(
            "{}\t{}\n",
            &self.timestamp.to_rfc3339_opts(SecondsFormat::Secs, true),
            &self.content
        )
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
}

impl fmt::Display for Entry {
    /// Formats a tweet for display in terminal.
    /// Alternate format uses absolute time.
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let timestamp = if formatter.alternate() {
            self.timestamp
                .with_timezone(&Local)
                .to_rfc3339_opts(SecondsFormat::Secs, true)
        } else {
            Self::format_duration(Local::now() - self.timestamp.with_timezone(&Local))
        };

        write!(
            formatter,
            "\n@{} {}\n{}",
            self.author.as_ref().unwrap_or(&"".to_string()),
            &timestamp,
            &self.content
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_duration() {
        assert_eq!(
            Entry::format_duration(Duration::days(365 * 2)),
            "2 years ago"
        );
        assert_eq!(Entry::format_duration(Duration::days(365)), "1 year ago");
        assert_eq!(
            Entry::format_duration(Duration::days(30 * 3)),
            "3 months ago"
        );
        assert_eq!(Entry::format_duration(Duration::days(30)), "1 month ago");
        assert_eq!(Entry::format_duration(Duration::weeks(4)), "4 weeks ago");
        assert_eq!(Entry::format_duration(Duration::weeks(1)), "1 week ago");
        assert_eq!(Entry::format_duration(Duration::days(4)), "4 days ago");
        assert_eq!(Entry::format_duration(Duration::days(1)), "1 day ago");
        assert_eq!(Entry::format_duration(Duration::hours(23)), "23 hours ago");
        assert_eq!(Entry::format_duration(Duration::hours(1)), "1 hour ago");
        assert_eq!(
            Entry::format_duration(Duration::minutes(5)),
            "5 minutes ago"
        );
        assert_eq!(Entry::format_duration(Duration::minutes(1)), "1 minute ago");
        assert_eq!(Entry::format_duration(Duration::seconds(30)), "just now");
    }
}
