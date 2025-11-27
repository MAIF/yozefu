use std::time;

use app::configuration::TimestampFormat;
use chrono::{DateTime, Utc};
use lib::KafkaRecord;
use ratatui::{
    style::Stylize,
    text::{Line, Span},
};

use crate::Theme;

#[inline]
pub(crate) fn colorize_timestamp<'a>(
    kafka_record: &KafkaRecord,
    theme: &Theme,
    timestamp_format: &TimestampFormat,
) -> Line<'a> {
    if timestamp_format == &TimestampFormat::Ago {
        let timestamp_in_millis = kafka_record.timestamp.unwrap_or(0);
        let published_at = DateTime::from_timestamp_millis(timestamp_in_millis).unwrap();
        let ago_formatter = timeago::Formatter::new();
        let duration = (Utc::now() - published_at)
            .to_std()
            .unwrap_or(time::Duration::ZERO);

        return Line::from(vec![Span::from(ago_formatter.convert(duration))]);
    }

    match kafka_record.timestamp_as_local_date_time() {
        Some(t) => {
            let formatted_date_time = t.to_rfc3339_opts(chrono::SecondsFormat::Millis, false);

            let date = &formatted_date_time[..10]; // "2025-05-12"
            let time = &formatted_date_time[11..23]; // "13:45:30.123"
            let tz = &formatted_date_time[24..]; // "+02:00"
            let date = date.to_string();
            Line::from(vec![
                Span::from(date.to_string()).fg(theme.magenta),
                Span::from("T").fg(theme.red),
                Span::from(time.to_string()).fg(theme.blue),
                Span::from("+"),
                Span::from(tz.to_string()).fg(theme.cyan),
            ])
        }
        None => Line::from(""),
    }
}

pub(crate) fn colorize_key<'a>(key: &str, theme: &Theme, width: usize) -> Line<'a> {
    let pouet = match key.len() > width {
        true => {
            vec![Span::from(format!("{}..", &key[0..width - 2]))]
        }
        false => {
            vec![Span::from(key.to_string())]
        }
    };
    Line::from(pouet).fg(theme.green)
}

pub(crate) fn colorize_and_shorten_topic<'a>(
    topic: &str,
    partition: i32,
    theme: &Theme,
    width: usize,
) -> Line<'a> {
    let partition_line = Line::from(vec![
        "[".into(),
        partition.to_string().fg(theme.yellow),
        "]".into(),
    ]);

    let partition_width = partition_line.width();
    let topic = match ((topic.len() + partition_width) as isize) - (width as isize) {
        1 => format!(".{}", &topic[topic.len() - (width - partition_width - 1)..]),
        i if i <= 0 => topic.to_string(),
        _ => format!(
            "..{}",
            &topic[topic.len() - (width - partition_width - 2)..]
        ),
    };

    let mut pp = vec![Span::from(topic.to_string())];
    pp.extend(partition_line);
    Line::from(pp)
}

#[test]
fn test_shorten_topic() {
    assert_eq!(
        colorize_and_shorten_topic("test-topic", 0, &Theme::light(), 10).to_string(),
        "..topic[0]"
    );
}

#[test]
fn test_shorten_topic_long_topic() {
    assert_eq!(
        colorize_and_shorten_topic("a-very-long-topic-to-shorten", 0, &Theme::light(), 10)
            .to_string(),
        "..orten[0]"
    );

    assert_eq!(
        colorize_and_shorten_topic("a-very-long-topic-to-shorten", 32, &Theme::light(), 10)
            .to_string(),
        "..rten[32]"
    );
}
