use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::{tag, take_until},
    combinator::map,
    sequence::delimited,
};

/// A string is delimited by single or double quotes.
/// ```text
/// "this is a string"
/// 'this a another string'
/// ```
pub(crate) fn parse_string(input: &str) -> IResult<&str, String> {
    map(
        alt((
            delimited(tag("\""), take_until("\""), tag("\"")),
            delimited(tag("'"), take_until("'"), tag("'")),
        )),
        |d: &str| d.to_string(),
    )
    .parse(input)
}

#[test]
fn test_parse_string() {
    assert_eq!(parse_string(r#"'halo'"#), Ok(("", "halo".to_string())));
    assert_eq!(parse_string(r#""hola""#), Ok(("", "hola".to_string())));
}
