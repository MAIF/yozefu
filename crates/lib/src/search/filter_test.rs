use crate::search::filter::{Filter, Parameter};

#[test]
fn test_filter_to_string() {
    assert_eq!(
        Filter {
            name: "my_filter".into(),
            parameters: vec![Parameter::Number(10), Parameter::String("value".into())],
        }
        .to_string(),
        "my_filter(10, 'value')"
    )
}
