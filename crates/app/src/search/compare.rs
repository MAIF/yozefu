use lib::{
    kafka::Comparable,
    search::{
        compare::{CompareExpression, NumberOperator, StringOperator},
        filter::Filter,
        offset::FromOffset,
    },
};

use super::SearchContext;
use crate::search::Search;

impl Search for CompareExpression {
    fn offset(&self) -> Option<FromOffset> {
        match self {
            CompareExpression::Offset(NumberOperator::Equal, e) => Some(FromOffset::Offset(*e)),
            CompareExpression::Offset(NumberOperator::GreaterOrEqual, e) => {
                Some(FromOffset::Offset(*e))
            }
            CompareExpression::Offset(NumberOperator::GreaterThan, e) => {
                Some(FromOffset::Offset(*e + 1))
            }
            CompareExpression::OffsetTail(e) => Some(FromOffset::OffsetTail(*e)),
            CompareExpression::Timestamp(op, e) => match op {
                NumberOperator::GreaterThan => Some(FromOffset::Timestamp(e.timestamp_millis())),
                NumberOperator::GreaterOrEqual => {
                    Some(FromOffset::Timestamp(e.timestamp_millis() - 1000))
                }
                NumberOperator::Equal => Some(FromOffset::Timestamp(e.timestamp_millis() - 1000)),
                NumberOperator::LowerThan => None,
                NumberOperator::LowerOrEqual => None,
                _ => None,
            },
            _ => None,
        }
    }

    fn matches(&self, context: &SearchContext) -> bool {
        let record = context.record;
        match self {
            CompareExpression::OffsetTail(_) => true,
            CompareExpression::Partition(op, p) => match op {
                &NumberOperator::GreaterThan => record.partition > *p,
                NumberOperator::GreaterOrEqual => record.partition >= *p,
                NumberOperator::LowerThan => record.partition < *p,
                NumberOperator::LowerOrEqual => record.partition <= *p,
                NumberOperator::Equal => record.partition == *p,
                NumberOperator::NotEqual => record.partition != *p,
            },
            CompareExpression::Offset(op, p) => match op {
                NumberOperator::GreaterThan => record.offset > *p,
                NumberOperator::GreaterOrEqual => record.offset >= *p,
                NumberOperator::LowerThan => record.offset < *p,
                NumberOperator::LowerOrEqual => record.offset <= *p,
                NumberOperator::Equal => record.offset == *p,
                NumberOperator::NotEqual => record.offset != *p,
            },
            CompareExpression::Topic(op, t) => match op {
                StringOperator::Equal => record.topic == *t,
                StringOperator::NotEqual => record.topic != *t,
                StringOperator::Contain => record.topic.contains(t),
                StringOperator::StartWith => record.topic.starts_with(t),
            },
            CompareExpression::Size(op, s) => match op {
                NumberOperator::GreaterThan => record.size > *s as usize,
                NumberOperator::GreaterOrEqual => record.size >= *s as usize,
                NumberOperator::LowerThan => record.size < *s as usize,
                NumberOperator::LowerOrEqual => record.size <= *s as usize,
                NumberOperator::Equal => record.size == *s as usize,
                NumberOperator::NotEqual => record.size != *s as usize,
            },
            CompareExpression::Key(op, t) => record.key.compare(&None, op, t),
            CompareExpression::Value(left, op, t) => record.value.compare(left, op, t),
            CompareExpression::Header(left, op, t) => {
                let headers = &record.headers;
                let header = headers.get(left);
                if header.is_none() {
                    return false;
                }
                let header = header.unwrap();
                match op {
                    StringOperator::Contain => header.contains(t),
                    StringOperator::Equal => header == t,
                    StringOperator::StartWith => header.starts_with(t),
                    StringOperator::NotEqual => header != t,
                }
            }
            CompareExpression::Timestamp(op, t) => {
                let ts = record.timestamp_as_local_date_time().unwrap();
                match op {
                    NumberOperator::GreaterThan => ts > *t,
                    NumberOperator::GreaterOrEqual => ts >= *t,
                    NumberOperator::LowerThan => ts < *t,
                    NumberOperator::LowerOrEqual => ts <= *t,
                    NumberOperator::Equal => ts == *t,
                    NumberOperator::NotEqual => ts != *t,
                }
            }
            CompareExpression::TimestampBetween(from, to) => {
                let ts = record.timestamp_as_local_date_time().unwrap();
                from <= &ts && &ts <= to
            }
        }
    }

    fn filters(&self) -> Vec<Filter> {
        vec![]
    }
}

#[test]
fn test_matches() {
    use crate::search::filter::CACHED_FILTERS;
    use lib::kafka::KafkaRecord;
    use std::path::PathBuf;

    let compare = CompareExpression::Offset(NumberOperator::Equal, 42);
    let record = KafkaRecord {
        topic: "test-topic".to_string(),
        partition: 0,
        offset: 42,
        key: lib::DataType::String("key".to_string()),
        value: lib::DataType::String("value".to_string()),
        timestamp: None,
        headers: std::collections::BTreeMap::new(),
        key_schema: None,
        value_schema: None,
        size: 12,
        key_as_string: "key".to_string(),
        value_as_string: "value".to_string(),
    };
    let context = SearchContext {
        record: &record,
        filters: &CACHED_FILTERS,
        filters_directory: PathBuf::from("."),
    };

    assert!(compare.matches(&context))
}
