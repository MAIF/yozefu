use lib::KafkaRecord;
use ratatui::{
    style::Stylize,
    text::{Line, Span},
};

use crate::Theme;

#[inline]
pub(crate) fn colorize_timestamp<'a>(kafka_record: &KafkaRecord, theme: &Theme) -> Line<'a> {
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

pub(crate) fn colorize_key<'a>(key: &str, theme: &Theme) -> Line<'a> {
    let pouet = match key.len() > 11 {
        true => {
            vec![Span::from(format!("{}..", &key[0..8]))]
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
) -> Line<'a> {
    let partition = Line::from(vec![
        "[".into(),
        partition.to_string().fg(theme.yellow),
        "]".into(),
    ]);

    let partition_length = partition.width();
    let pouet = match topic.len() + partition_length > 12 {
        true => {
            let remaining = 12_usize.saturating_sub(partition_length).saturating_sub(6);
            let mut pp = vec![Span::from(format!(
                "{}..{}",
                &topic[0..4],
                &topic[topic.len() - remaining..]
            ))];
            pp.extend(partition);
            pp
        }
        false => {
            let mut pp = vec![Span::from(topic.to_string())];
            pp.extend(partition);
            pp
        }
    };
    Line::from(pouet)
}

#[test]
fn test_shorten_topic() {
    assert_eq!(
        colorize_and_shorten_topic("test-topic", 0, &Theme::light()).to_string(),
        "test..pic[0]"
    );
}

#[test]
fn test_shorten_topic_long_topic() {
    assert_eq!(
        colorize_and_shorten_topic("a-very-long-topic-to-shorten", 0, &Theme::light()).to_string(),
        "a-ve..ten[0]"
    );

    assert_eq!(
        colorize_and_shorten_topic("a-very-long-topic-to-shorten", 32, &Theme::light()).to_string(),
        "a-ve..en[32]"
    );
}
