#![no_main]
/// This filter makes a http call to "https://mcdostone.github.io/".
/// It always returns a match.
/// 
/// the goal of this search filter is to test that no wasm module can access the internet.
/// It it used in the test `...``
use extism_pdk::*;
use json::Value;

use yozefu_wasm_types::{FilterInput, FilterResult};

#[plugin_fn]
pub fn matches(input: Json<FilterInput>) -> FnResult<Json<FilterResult>> {
    let request = HttpRequest::new("https://mcdostone.github.io/");;
    let res = http::request::<()>(&request, None)?;
    Ok(Json(FilterResult {
        r#match: true
    }))
}

#[plugin_fn]
pub fn parse_parameters(params: Json<Vec<Value>>) -> FnResult<()> {
    Ok(())
}
