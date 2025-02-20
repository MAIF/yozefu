use std::fmt::Display;

use nom::{
    IResult, Parser, branch::alt, bytes::complete::tag, combinator::map, sequence::preceded,
};

use super::{
    atom::{Atom, parse_atom},
    wsi::wsi,
};

/// A term is either:
///  - An atom,
///  - Or a negative atom.
///```sql
/// !(offset > 50)
/// offset > 50
/// ```
#[derive(Debug, PartialEq, Clone)]
pub enum Term {
    Not(Atom),
    Atom(Atom),
}

pub(crate) fn parse_term(input: &str) -> IResult<&str, Term> {
    alt((
        map(preceded(wsi(tag("!")), parse_atom), Term::Not),
        map(parse_atom, Term::Atom),
    ))
    .parse(input)
}

impl Display for Term {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Term::Not(a) => write!(f, "!{}", a),
            Term::Atom(a) => write!(f, "{}", a),
        }
    }
}

//#[test]
//fn test_parse() {
//    use crate::search::symbol::Symbol;
//    assert_eq!(
//        parse_term(r#"!partition"#),
//        Ok(("", Term::Not(Atom::Symbol(Symbol::Partition))))
//    );
//    assert_eq!(
//        parse_term(r#"topic"#),
//        Ok(("", Term::Atom(Atom::Symbol(Symbol::Topic))))
//    );
//}
