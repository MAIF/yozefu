function matches() {
  // TODO - Edit the code as per your requirements
  const input = JSON.parse(Host.inputString())
  const firstParam = input.params[0];
  const key = input.record.key;
  Host.outputString(JSON.stringify({ match : key.ends_with(firstParam) }));
  return 0
}

function parse_parameters() {
  const params = JSON.parse(Host.inputString())
  // TODO - Edit the code as per your requirements
  let length = params.length
  if(length != 1) {
    throw Error(`This search filter expects a string argument. Found %v arguments`, length);
  }
  return 0
}

module.exports = { matches, parse_parameters };
