use std::{
    collections::HashMap,
    sync::{LazyLock, Mutex},
};

use extism::{Plugin, convert::Json};
use itertools::Itertools;
use lib::{
    FilterResult,
    search::filter::{Filter, FilterInput},
};
use tracing::error;

use super::{Search, SearchContext};

pub const MATCHES_FUNCTION_NAME: &str = "matches";
pub const PARSE_PARAMETERS_FUNCTION_NAME: &str = "parse_parameters";

/// FILTERS are lazy loaded and cached in memory.
pub(crate) static CACHED_FILTERS: LazyLock<Mutex<HashMap<String, Plugin>>> =
    LazyLock::new(|| HashMap::new().into());

impl Search for Filter {
    fn matches(&self, context: &SearchContext) -> bool {
        let mut filters = context.filters.lock().unwrap();
        let plugin = &mut filters.get_mut(&self.name).unwrap();
        let input = FilterInput {
            record: context.record.clone(),
            params: self.parameters.iter().map(|e| e.json()).collect_vec(),
        };

        match plugin
            .call::<String, Json<FilterResult>>(
                MATCHES_FUNCTION_NAME,
                serde_json::to_string(&input).unwrap(),
            )
            .map(|e| e.0)
        {
            Ok(res) => res.r#match,
            Err(e) => {
                error!(
                    "Error when calling '{MATCHES_FUNCTION_NAME}' from wasm module '{}': {e:?}",
                    self.name
                );
                false
            }
        }
    }

    fn filters(&self) -> Vec<Filter> {
        vec![]
    }
}
