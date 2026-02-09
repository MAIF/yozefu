use apache_avro::types::Value;
use serde_json::{Map, Number};

/// Converts an Avro value to a JSON value.
pub(crate) fn avro_to_json(value: Value) -> serde_json::Value {
    match value {
        Value::Null => serde_json::Value::Null,
        Value::Boolean(b) => serde_json::Value::Bool(b),
        Value::Int(i) => serde_json::Value::Number(Number::from(i)),
        Value::Long(l) => serde_json::Value::Number(Number::from(l)),
        Value::Float(f) => parse_number(f.into()),
        Value::Double(f) => parse_number(f),
        Value::Bytes(vec) => serde_json::Value::Array(
            vec.iter()
                .map(|b| serde_json::Value::Number(Number::from(*b)))
                .collect(),
        ),
        Value::String(s) => serde_json::Value::String(s),
        Value::Fixed(_, vec) => serde_json::Value::Array(
            vec.iter()
                .map(|b| serde_json::Value::Number(Number::from(*b)))
                .collect(),
        ),
        Value::Enum(_, s) => serde_json::Value::String(s),
        Value::Union(_, value) => avro_to_json(*value),
        Value::Array(vec) => {
            serde_json::Value::Array(vec.iter().map(|v| avro_to_json(v.clone())).collect())
        }
        Value::Map(hash_map) => serde_json::Value::Object(
            hash_map
                .into_iter()
                .map(|(k, v)| (k, avro_to_json(v)))
                .collect(),
        ),
        Value::Record(vec) => {
            serde_json::Value::Object(vec.into_iter().map(|(k, v)| (k, avro_to_json(v))).collect())
        }
        Value::Date(date) => serde_json::Value::Number(Number::from(date)),
        Value::TimeMillis(ts) => serde_json::Value::Number(Number::from(ts)),
        Value::TimeMicros(ts) => serde_json::Value::Number(Number::from(ts)),
        Value::TimestampMillis(ts) => serde_json::Value::Number(Number::from(ts)),
        Value::TimestampMicros(ts) => serde_json::Value::Number(Number::from(ts)),
        Value::TimestampNanos(ts) => serde_json::Value::Number(Number::from(ts)),
        Value::LocalTimestampMillis(ts) => serde_json::Value::Number(Number::from(ts)),
        Value::LocalTimestampMicros(ts) => serde_json::Value::Number(Number::from(ts)),
        Value::LocalTimestampNanos(ts) => serde_json::Value::Number(Number::from(ts)),
        Value::Uuid(uuid) => serde_json::Value::String(uuid.to_string()),
        Value::Duration(duration) => {
            let mut map = Map::with_capacity(3);
            let i: u32 = duration.months().into();
            map.insert("months".to_string(), serde_json::Value::Number(i.into()));
            let i: u32 = duration.millis().into();
            map.insert("millis".to_string(), serde_json::Value::Number(i.into()));
            let i: u32 = duration.days().into();
            map.insert("days".to_string(), serde_json::Value::Number(i.into()));
            serde_json::Value::Object(map)
        }
        Value::Decimal(_decimal) => serde_json::Value::String(
            "Yozefu error: I don't know how to encode a decimal to json. It fails silently"
                .to_string(),
        ),
        Value::BigDecimal(big_decimal) => serde_json::Value::String(big_decimal.to_string()),
    }
}

/// Parses a floating-point number into a JSON value, handling special cases for NaN and infinity.
/// Bug discovered in <https://github.com/MAIF/yozefu/issues/239>
fn parse_number(number: f64) -> serde_json::Value {
    match Number::from_f64(number) {
        Some(num) => serde_json::Value::Number(num),
        None => match number.classify() {
            std::num::FpCategory::Nan => serde_json::Value::String("NaN".to_string()),
            std::num::FpCategory::Infinite if number.is_sign_positive() => {
                serde_json::Value::String("Infinity".to_string())
            }
            std::num::FpCategory::Infinite if number.is_sign_negative() => {
                serde_json::Value::String("-Infinity".to_string())
            }
            _ => unreachable!(
                "The only way from_f64 can return None is if the float is NaN or infinite, which we handle above."
            ),
        },
    }
}

#[test]
fn test_avro_to_json() {
    assert_eq!(
        avro_to_json(Value::Int(42)),
        serde_json::Value::Number(Number::from(42))
    );

    assert_eq!(
        avro_to_json(Value::Float(f32::NAN)),
        serde_json::Value::String("NaN".to_string())
    );

    assert_eq!(
        avro_to_json(Value::Float(f32::INFINITY)),
        serde_json::Value::String("Infinity".to_string())
    );

    assert_eq!(
        avro_to_json(Value::Float(-f32::INFINITY)),
        serde_json::Value::String("-Infinity".to_string())
    );
}
