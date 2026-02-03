---
description: Creating custom search filters using WebAssembly modules.
---

# Creating a search filter

Let's say you want to list all kafka records where the key ends with `1234`. 
Currently, the query syntax doesn't offer such feature. Fortunately, you have the ability to extend the search engine and create your own search logic.

You can implement what it's called **a search filter**. A search filter is a WebAssembly module that exports a `parse_parameters` and `matches` functions. You can pass string or number parameters to that function. It will look like this:

```sql
-- from the beginning of the topic, retrieve all records where the key ends with '1234'
from begin where key-ends-with('1234')
```

The name of the function corresponds to the name of the wasm file. In the example above, the wasm file is `key-ends-with.wasm`. 

> [!TIP]
> Wasm files can be found at `yozf config get filters-dir`.



## Defining your search filter


Yōzefu relies on [Extism](https://extism.org/) to develop and execute search filters.
The WebAssembly module we're going to implement must export 2 functions, `parse_parameters` and `matches`.

The first step is to choose your preferred programming language. Extism supports different programming languages. You can read more at [Extism Quickstart Guide](https://extism.org/docs/quickstart/plugin-quickstart/). I'll choose [golang](https://github.com/MAIF/yozefu/blob/main/crates/wasm-blueprints/golang) for this example. A [Rust example](https://github.com/MAIF/yozefu/blob/main/crates/wasm-blueprints/rust) is also available.

```bash
yozf create-filter --language golang key-ends-with --directory /tmp/my-filter

$EDITOR /tmp/my-filter
```

If you need more context about how WebAssembly is called from the Rust codebase, feel free to explore [filter.rs](https://github.com/MAIF/yozefu/blob/main/crates/app/src/search/filter.rs).

### Function `parse_parameters`

This function ensures that user-provided parameters are valid. This function is called once at parsing time.
The function must return a status code `0` when it's valid. Another status code means the parameters are invalid.

```go
// golang example
// https://github.com/MAIF/yozefu/blob/main/crates/wasm-blueprints/golang/main.go

//export parse_parameters
func ParseParameters() int32 {
	var params FilterParams
	err := pdk.InputJSON(&params)
	if err != nil {
		pdk.SetError(err)
		return 1
	}
	if len(params) != 1 {
		pdk.SetError(errors.New(fmt.Sprintf("This search filter expects a string argument. Found %v arguments", len(params))))
		return 2
	}
	return 0
}
```



### Function `matches`

This function receives a [JSON object](./filter-input.json) containing both the kafka record and the function parameters. It returns the json `{"match": true}` when the record matches your query. The output is represented by the struct [`FilterResult`](https://github.com/MAIF/yozefu/blob/main/crates/lib/src/search/mod.rs#L80-L89). This function is called for every kafka record read.



```go
// golang example
// https://github.com/MAIF/yozefu/blob/main/crates/wasm-blueprints/golang/main.go

// export matches
func Matches() int32 {
	input := FilterInput{}
	err := pdk.InputJSON(&input)
	param := input.Params[0]
	if err != nil {
		pdk.SetError(err)
		return False
	}
	if strings.HasSuffix(input.Record.Key, param) {
		err = pdk.OutputJSON(FilterResult{Match: true})
		if err != nil {
			pdk.SetError(err)
			return 1
		}
		return 0
	}
	_ = pdk.OutputJSON(FilterResult{Match: false})
	return 0
}
```


### Build it

Now, it's time to compile it to WebAssembly:

```bash
# Make is not required but make things easier
make build
```

You can also implement and run tests:
```bash
$EDITOR ./tests/parameters.json
$EDITOR ./tests/match.json
$EDITOR ./tests/no-match.json
make test
```

Finally import your filter 🎉
```bash
yozf import-filter 'plugin.wasm' --name "key-ends-with"

yozf -c my-cluster --topics "my-topic" "from begin where key-ends-with('1234')"
```