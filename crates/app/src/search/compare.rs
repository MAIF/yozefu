use lib::{
    kafka::Comparable,
    search::{
        compare::{CompareExpression, NumberOperator, StringOperator},
        filter::Filter,
        offset::FromOffset,
    },
};

use crate::search::Search;

use super::SearchContext;

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
