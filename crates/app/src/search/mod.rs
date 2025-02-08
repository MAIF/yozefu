//! Module implementing the search logic

use std::{
    collections::HashMap,
    str::FromStr,
    sync::{LazyLock, Mutex},
};

use extism::{Manifest, Plugin, Wasm};
use filter::{CACHED_FILTERS, FILTERS_DIR, PARSE_PARAMETERS_FUNCTION_NAME};
use itertools::Itertools;
use lib::{
    parse_search_query,
    search::{filter::Filter, offset::FromOffset},
    KafkaRecord, SearchQuery,
};
use log::error;

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
    pub record: &'a KafkaRecord,
    pub filters: &'a LazyLock<Mutex<HashMap<String, Plugin>>>,
}

impl SearchContext<'_> {
    pub fn new(record: &KafkaRecord) -> SearchContext<'_> {
        SearchContext {
            record,
            filters: &CACHED_FILTERS,
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

impl FromStr for ValidSearchQuery {
    type Err = lib::Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let query = parse_search_query(input).map_err(lib::Error::Search)?.1;
        let filters = query.filters();
        let dir = FILTERS_DIR.lock().unwrap().clone();
        for filter in filters {
            let name = filter.name;
            let path = dir.join(format!("{}.wasm", &name));
            let url = Wasm::file(&path);
            let manifest = Manifest::new([url]);
            let mut filters = CACHED_FILTERS.lock().unwrap();
            if !filters.contains_key(&name) {
                match Plugin::new(manifest, [], true) {
                    Ok(plugin) => filters.insert(name.to_string(), plugin),
                    Err(err) => {
                        error!("No such file '{}': {}", path.display(), err);
                        return Err(lib::Error::Error(format!(
                            "Cannot find search filter '{}'",
                            name
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
                    "Error when calling '{}' from wasm module '{}': {:?}",
                    PARSE_PARAMETERS_FUNCTION_NAME, name, e
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
