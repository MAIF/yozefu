use lib::search::{atom::Atom, filter::Filter, offset::FromOffset};

use super::{Search, SearchContext};

impl Search for Atom {
    fn offset(&self) -> Option<FromOffset> {
        match self {
            Atom::Symbol(_) => None,
            Atom::Compare(c) => c.offset(),
            Atom::Parenthesis(c) => c.offset(),
            Atom::Filter(_) => None,
        }
    }

    fn matches(&self, context: &SearchContext) -> bool {
        match self {
            Atom::Symbol(_) => false,
            Atom::Compare(e) => e.matches(context),
            Atom::Parenthesis(e) => e.matches(context),
            Atom::Filter(f) => f.matches(context),
        }
    }

    fn filters(&self) -> Vec<Filter> {
        match self {
            Atom::Symbol(_) => vec![],
            Atom::Compare(e) => e.filters(),
            Atom::Parenthesis(e) => e.filters(),
            Atom::Filter(f) => vec![f.clone()],
        }
    }
}
