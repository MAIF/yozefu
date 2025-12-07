#![no_main]
/// This filter returns `true`
/// when the key of the kafka record ends with the user-specified string parameter
///
/// ```sql
/// key-ends-with("rust")
/// ```
///
/// This WebAssembly module relies on the [Extism SDK](https://extism.org/docs/quickstart/plugin-quickstart/)
use extism_pdk::*;
use json::Value;

use yozefu_wasm_types::{FilterInput, FilterResult};

#[plugin_fn]
pub fn matches(input: Json<FilterInput>) -> FnResult<Json<FilterResult>> {
    // TODO - Edit the code as per your requirements
    let first_param = input.0.params.first().unwrap().as_str().unwrap();
    let key = input.0.record.key.raw();

    Ok(Json(key.ends_with(first_param).into()))
}

#[plugin_fn]
/// This function checks if the input parameters are valid
pub fn parse_parameters(params: Json<Vec<Value>>) -> FnResult<()> {
    // TODO - Edit the code as per your requirements
    let length = params.0.len();
    if length != 1 {
        return Err(WithReturnCode::new(
            Error::msg(format!(
                "This search filter expects a string argument. Found {} arguments",
                &length.to_string()
            )),
            1,
        ));
    }
    if params.0.first().unwrap().is_string() {
        return Ok(());
    }
    return Err(WithReturnCode::new(
        Error::msg(format!(
            "This search filter expects argument 1 to be a string, found {}",
            json::to_string(params.0.first().unwrap()).unwrap()
        )),
        2,
    ));
}
