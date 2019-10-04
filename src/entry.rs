use chrono::{offset::TimeZone, DateTime, Duration, FixedOffset, Local, SecondsFormat};

use std::fmt;

#[derive(Ord, PartialOrd, PartialEq, Eq, Debug)]
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
            Self::format_duration(Local::now() - self.timestamp.with_timezone(&Local))
        } else {
            self.timestamp
                .with_timezone(&Local)
                .to_rfc3339_opts(SecondsFormat::Secs, true)
        };

        write!(
            formatter,
            "@{} {}\n{}",
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
    fn test_new() {
        let start: DateTime<FixedOffset> = Local::now().into();
        let entry = Entry::new("content".to_string(), Some("author".to_string()));
        let end: DateTime<FixedOffset> = Local::now().into();

        assert_eq!(start.cmp(&entry.timestamp), std::cmp::Ordering::Less);
        assert_eq!(end.cmp(&entry.timestamp), std::cmp::Ordering::Greater);
        assert_eq!(entry.content, "content");
        assert_eq!(entry.author, Some("author".to_string()));
    }

    #[test]
    fn test_parse() {
        assert_eq!(Entry::parse("This is not valid twtxt.\t", None), Err(()));
        assert_eq!(
            Entry::parse("2016-02-04T13:30:01+01:00\tThis is valid twtxt.", None),
            Ok(Entry {
                timestamp: FixedOffset::east(3600).ymd(2016, 02, 04).and_hms(13, 30, 1),
                author: None,
                content: "This is valid twtxt.".to_string(),
            })
        );
    }

    #[test]
    fn test_to_twtxt() {
        assert_eq!(
            Entry {
                timestamp: FixedOffset::east(3600)
                    .ymd(2016, 02, 04)
                    .and_hms_milli(13, 30, 1, 238),
                author: None,
                content: "Hello world!".to_string(),
            }
            .to_twtxt(),
            "2016-02-04T13:30:01+01:00\tHello world!\n"
        );
    }

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

    #[test]
    fn test_display() {
        let timestamp = (Local::now() - Duration::weeks(1)).into();
        let entry = Entry {
            timestamp,
            author: Some("anonymous".to_string()),
            content: "Hello world!".to_string(),
        };

        assert_eq!(
            format!("{:#}", entry),
            format!("@anonymous 1 week ago\nHello world!",)
        );

        assert_eq!(
            format!("{}", entry),
            format!(
                "@anonymous {}\nHello world!",
                timestamp
                    .with_timezone(&Local)
                    .to_rfc3339_opts(SecondsFormat::Secs, true)
            )
        );
    }
}
