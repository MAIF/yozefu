use insta::{assert_debug_snapshot, glob};
use std::fs;
use yozefu_lib::parse_search_query;

#[test]
fn test_search_queries() {
    super::fix_timezone();
    glob!("inputs/*.sql", |path| {
        let input = fs::read_to_string(path).unwrap();
        let input = input.trim();
        insta::with_settings!({
            description => input.replace("\n", " "),
            filters => vec![
            ("[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}\\.[0-9]{6,}\\+[0-9]{2}:[0-9]{2}", "[datetime]"),
        ]}, {
            assert_debug_snapshot!(parse_search_query(input));
        });
    });
}
