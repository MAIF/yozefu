//! Module implementing the search logic

use extism::{Manifest, Plugin, Wasm};
use filter::{CACHED_FILTERS, PARSE_PARAMETERS_FUNCTION_NAME};
use itertools::Itertools;
use lib::{
    KafkaRecord, SearchQuery, parse_search_query,
    search::{filter::Filter, offset::FromOffset},
};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{LazyLock, Mutex},
};
use tracing::error;

pub mod atom;
pub mod compare;
pub mod expression;
pub mod filter;
pub mod search_query;
pub mod term;

pub trait Search {
    /// Returns the offset from which the search should start.
    fn offset(&self) -> Option<FromOffset> {
        None
    }
    /// returns `true` if the record matches the search query.
    fn matches(&self, context: &SearchContext) -> bool;

    /// Returns the search filters that are used in the search query.
    fn filters(&self) -> Vec<Filter>;
}

/// Struct that holds the context of the search.
/// It contains the record that is being searched and the loaded search filters.
pub struct SearchContext<'a> {
    /// The record that is being searched.
    pub record: &'a KafkaRecord,
    /// The search filters that are already loaded in memory.
    pub filters: &'a LazyLock<Mutex<HashMap<String, Plugin>>>,
    /// The directory containing the search filters
    pub filters_directory: PathBuf,
}

impl SearchContext<'_> {
    pub fn new<'a>(record: &'a KafkaRecord, filters_directory: &'a Path) -> SearchContext<'a> {
        SearchContext {
            record,
            filters: &CACHED_FILTERS,
            filters_directory: filters_directory.to_path_buf(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ValidSearchQuery(SearchQuery);

impl ValidSearchQuery {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn limit(&self) -> Option<usize> {
        self.0.limit
    }

    pub fn query(&self) -> &SearchQuery {
        &self.0
    }
}

impl ValidSearchQuery {
    pub fn from(input: &str, filters_directory: &Path) -> Result<Self, lib::Error> {
        let query = parse_search_query(input).map_err(lib::Error::Search)?.1;
        let filters = query.filters();
        for filter in filters {
            let name = filter.name;
            let path = filters_directory.join(format!("{}.wasm", &name));
            let url = Wasm::file(&path);
            let manifest = Manifest::new([url]);
            let mut filters = CACHED_FILTERS.lock().unwrap();
            if !filters.contains_key(&name) {
                match Plugin::new(manifest, [], true) {
                    Ok(plugin) => filters.insert(name.to_string(), plugin),
                    Err(err) => {
                        error!("No such file '{}': {}", path.display(), err);
                        return Err(lib::Error::Error(format!(
                            "Cannot find search filter '{name}' in {}: {}",
                            path.parent().unwrap().display(),
                            err
                        )));
                    }
                };
            }
            let params = filter.parameters;
            let wasm_module = &mut filters.get_mut(&name).unwrap();
            if let Err(e) = wasm_module.call::<&str, &str>(
                PARSE_PARAMETERS_FUNCTION_NAME,
                &serde_json::to_string(&params.iter().map(|e| e.json()).collect_vec()).unwrap(),
            ) {
                error!(
                    "Error when calling '{PARSE_PARAMETERS_FUNCTION_NAME}' from wasm module '{name}': {e:?}"
                );
                return Err(lib::Error::Error(format!("{}: {e}", &name)));
            };
        }

        Ok(ValidSearchQuery(query))
    }
}

impl Search for ValidSearchQuery {
    /// Returns the offset from which the search should start.
    fn offset(&self) -> Option<FromOffset> {
        self.0.offset()
    }

    fn matches(&self, context: &SearchContext) -> bool {
        self.0.matches(context)
    }

    fn filters(&self) -> Vec<Filter> {
        self.0.filters()
    }
}

#[cfg(test)]
mod tests {
    use lib::DataType;

    use super::*;

    #[test]
    fn test_search_query_must_match() {
        let filters_directory = PathBuf::from("tests/filters");
        let input = "from begin";
        let query = ValidSearchQuery::from(input, &filters_directory).unwrap();

        let record = KafkaRecord {
            key: DataType::String("".into()),
            value: DataType::String("".into()),
            ..Default::default()
        };

        let context = SearchContext::new(&record, &filters_directory);
        assert!(query.matches(&context));
    }

    #[test]
    fn unknown_search_filter() {
        let filters_directory = PathBuf::from("tests/filters");
        let input = "from begin my_filter()";
        assert!(ValidSearchQuery::from(input, &filters_directory).is_err())
    }

    #[test]
    #[ignore]
    fn test_wasm_should_not_have_access_to_network() {
        use tracing::Level;
        testing_logger::setup();

        let filters_directory = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("http_search_filter");
        let input = "from begin module()";
        let query = ValidSearchQuery::from(input, &filters_directory).unwrap();

        let record = KafkaRecord {
            key: DataType::String("".into()),
            value: DataType::String("".into()),
            ..Default::default()
        };

        let context = SearchContext::new(&record, &filters_directory);
        assert!(!query.matches(&context));

        testing_logger::validate(|captured_logs| {
            let logs = captured_logs
                .iter()
                .filter(|c| c.level.to_string() == Level::ERROR.to_string())
                .collect::<Vec<&testing_logger::CapturedLog>>();
            assert_eq!(2, logs.len());
            assert!(
                logs[0]
                    .body
                    .contains("HTTP request to https://mcdostone.github.io/ is not allowed")
            );
        });
    }

    #[test]
    fn test_matches_with_fine_grained_filter_on_json_field() {
        use crate::search::filter::CACHED_FILTERS;
        use lib::kafka::KafkaRecord;
        use serde_json::json;
        use std::path::PathBuf;

        let filters_directory = PathBuf::from(".");

        let query = ValidSearchQuery::from(
            r#"from end - 10 value.myInteger == "42""#,
            &filters_directory,
        )
        .unwrap();
        let record = KafkaRecord {
            topic: "test-topic".to_string(),
            partition: 0,
            offset: 42,
            key: lib::DataType::String("key".to_string()),
            value: lib::DataType::Json(json!({"myInteger": 42})),
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
            filters_directory,
        };

        assert!(query.matches(&context))
    }
}
