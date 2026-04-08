use criterion::{Criterion, criterion_group, criterion_main};
use fake::{Fake, uuid::UUIDv7};
use lib::{DataType, KafkaRecord, SearchQuery};
use mock_json::mock;
use serde_json::json;
use std::hint::black_box;
use std::{collections::BTreeMap, env::temp_dir};
use yozefu_app::search::{Search, SearchContext};

fn generate_mock_value() -> serde_json::Value {
    mock(&json!({
        "code":0,
        "msg":"just text",
        "data":[{
            "id":"@Id|10",
            "title": "@Title",
            "datetime":"@DateTime",
            "author":{
                "name":"@Name",
                "id":"@Guid",
                "email":"@Email",
                "id_number":"@IdNumber",
                "ip":"@Ip",
                "phones":["@Phone", 1, 3],
                "blog":"@Url",
                "avatar":"@Image|80X80|f7f7f7|fff"
            }
        }, 10, 50]
    }))
}

fn generate_mock_key() -> serde_json::Value {
    mock(&json!({
        "code":0,
        "msg":"just text",
        "data":[{
            "id":"@Id|10",
            "title": "@Title",
            "datetime":"@DateTime",
            "author":{
                "name":"@Name",
                "id":"@Guid",
                "email":"@Email",
                "id_number":"@IdNumber",
                "ip":"@Ip",
                "phones":["@Phone", 1, 3],
                "blog":"@Url",
                "avatar":"@Image|80X80|f7f7f7|fff"
            }
        }, 10, 50]
    }))
}

fn generate_mock_kafka_record() -> KafkaRecord {
    let topic: String = UUIDv7.fake();
    let key: String = UUIDv7.fake();
    let value = generate_mock_value();
    let value_as_string = serde_json::to_string(&value).unwrap();

    KafkaRecord {
        key: DataType::String(key.clone()),
        value: DataType::Json(value),
        timestamp: Some((0..i64::MAX).fake::<i64>()),
        topic,
        partition: (0..16).fake::<i32>(),
        offset: (0..100_000_000).fake::<i64>(),
        headers: BTreeMap::new(),
        key_schema: None,
        value_schema: None,
        size: (0..18000).fake::<usize>(),
        key_as_string: serde_json::to_string(&key).unwrap(),
        value_as_string,
    }
}

fn read_all(c: &mut Criterion) {
    let filter_dir = temp_dir();
    let (_, search_query) = SearchQuery::parse("from begin").unwrap();
    c.bench_function("read all records", |b| {
        b.iter(|| {
            //thread::sleep(std::time::Duration::from_millis(2000));
            let record = generate_mock_kafka_record();
            let context = SearchContext::new(&record, &filter_dir);
            let _ = search_query.matches(black_box(&context));
        })
    });
}

fn even_offset(c: &mut Criterion) {
    let filter_dir = temp_dir();
    let (_, search_query) = SearchQuery::parse("from begin partition == 10").unwrap();
    c.bench_function("partition == 10", |b| {
        b.iter(|| {
            //thread::sleep(std::time::Duration::from_millis(2000));
            let record = generate_mock_kafka_record();
            let context = SearchContext::new(&record, &filter_dir);
            let _ = search_query.matches(black_box(&context));
        })
    });
}

fn value_contains_string(c: &mut Criterion) {
    let filter_dir = temp_dir();
    let (_, search_query) =
        SearchQuery::parse("from begin value contains 'fff' or key contains '34'").unwrap();
    c.bench_function("value contains 'fff' or value key contains '34'", |b| {
        b.iter(|| {
            //thread::sleep(std::time::Duration::from_millis(2000));
            let record = generate_mock_kafka_record();
            let context = SearchContext::new(&record, &filter_dir);
            let _ = search_query.matches(black_box(&context));
        })
    });
}

criterion_group!(benches, read_all, even_offset, value_contains_string);
criterion_main!(benches);
