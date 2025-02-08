use lib::search::{filter::Filter, offset::FromOffset, term::Term};

use super::{Search, SearchContext};

impl Search for Term {
    fn offset(&self) -> Option<FromOffset> {
        match self {
            Term::Not(_) => None,
            Term::Atom(a) => a.offset(),
        }
    }

    fn matches(&self, context: &SearchContext) -> bool {
        match self {
            Term::Not(a) => !a.matches(context),
            Term::Atom(a) => a.matches(context),
        }
    }

    fn filters(&self) -> Vec<Filter> {
        match self {
            Term::Not(atom) => atom.filters(),
            Term::Atom(atom) => atom.filters(),
        }
    }
}
