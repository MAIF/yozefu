package main

import (
	"errors"
	"fmt"
	"strings"

	"github.com/extism/go-pdk"
)

// This search filter shows how to write a search filter for Yozefu in Golang: https://extism.org/docs/quickstart/plugin-quickstart
//
// This Extism WebAssembly plugin filter Kafka records based on a key suffix.
// The plugin exports two functions:
// Matches: This function checks if the key of the Kafka record ends with the specified parameter.
//    It reads the input JSON, extracts the filter parameter, and compares it with the record's key.
//    If the key ends with the parameter, it returns a match result; otherwise, it returns no match.
// ParseParameters: This function verifies the number of parameters passed to the filter are valid.
//
// In Yozefu, the filter can be called like this: `from begin key-ends-with("my-suffix")`

// Representation of a kafka record in Golang
type KafkaRecord struct {
	Value     string      `json:"value"`
	Key       string      `json:"key"`
	Topic     string      `json:"topic"`
	Timestamp int64       `json:"timestamp"`
	Partition int         `json:"partition"`
	Offset    int         `json:"offset"`
	Headers   interface{} `json:"headers"`
}

// Here, this search filter accepts a list of string parameters.
type FilterParams = []string

// FilterInput represents the input to the filter
type FilterInput struct {
	Record KafkaRecord  `json:"record"`
	Params FilterParams `json:"params"`
}

// The output of the filter
type FilterResult struct {
	Match bool `json:"match"`
}

// Returns true when the key of the Kafka record ends with the user-provided parameter.
//
//export matches
func Matches() int32 {
	// TODO - Edit the code as per your requirements
	input := FilterInput{}
	err := pdk.InputJSON(&input)
	// Get the first parameter
	param := input.Params[0]
	if err != nil {
		pdk.SetError(err)
		return 1
	}
	// Check if the key ends with the parameter, return true if it does
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

// This filter accepts a single string parameter.
// If the number of parameters is not equal to 1, it returns an error.
//
//export parse_parameters
func ParseParameters() int32 {
	// TODO - Edit the code as per your requirements
	var params FilterParams
	err := pdk.InputJSON(&params)
	if err != nil {
		pdk.SetError(err)
		return 1
	}
	if len(params) != 1 {
		pdk.SetError(errors.New(fmt.Sprintf("This search filter expects a string argument. Found %v arguments", len(params))))
		return 1
	}
	return 0
}

// The main function is empty as it is not used in the Wasm plugin context.
func main() {}
