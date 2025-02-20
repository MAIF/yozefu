use nom::{
    AsChar, Compare, Input, Parser,
    branch::alt,
    bytes::complete::tag,
    character::complete::{line_ending, space1},
    combinator::value,
    error::ParseError,
    multi::many0,
    sequence::preceded,
};

/// Gets rid of spaces, tabs and backslash + newline.
pub(crate) fn wsi<I, F, O, E: ParseError<I>>(inner: F) -> impl Parser<I, Output = O, Error = E>
where
    I: Clone + Input,
    I: Compare<&'static str>,
    <I as Input>::Item: AsChar,
    F: Parser<I, Output = O, Error = E>,
{
    preceded(
        value(
            (),
            many0(alt((preceded(tag("\\"), line_ending), line_ending, space1))),
        ),
        inner,
    )
}
