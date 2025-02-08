use lib::search::{
    expression::{AndExpression, OrExpression},
    filter::Filter,
    offset::FromOffset,
};

use super::{Search, SearchContext};

impl Search for AndExpression {
    fn offset(&self) -> Option<FromOffset> {
        match self {
            Self::AndTerm(t) => t.offset(),
            Self::AndExpression(v) => {
                for vv in v {
                    let o = vv.offset();
                    if o.is_some() {
                        return o;
                    }
                }
                None
            }
        }
    }

    fn matches(&self, context: &SearchContext) -> bool {
        let record = context;
        match self {
            Self::AndTerm(t) => t.matches(record),
            Self::AndExpression(e) => {
                for ee in e {
                    if !ee.matches(record) {
                        return false;
                    }
                }
                true
            }
        }
    }

    fn filters(&self) -> Vec<Filter> {
        match self {
            AndExpression::AndTerm(term) => term.filters(),
            AndExpression::AndExpression(vec) => vec.iter().flat_map(|t| t.filters()).collect(),
        }
    }
}

impl Search for OrExpression {
    fn offset(&self) -> Option<FromOffset> {
        match self {
            Self::OrTerm(t) => t.offset(),
            Self::OrExpression(_) => None,
        }
    }

    fn matches(&self, context: &SearchContext) -> bool {
        match self {
            Self::OrTerm(t) => t.matches(context),
            Self::OrExpression(e) => {
                if e.is_empty() {
                    return true;
                }
                for ee in e {
                    if ee.matches(context) {
                        return true;
                    }
                }
                false
            }
        }
    }

    fn filters(&self) -> Vec<Filter> {
        match self {
            OrExpression::OrTerm(and_expression) => and_expression.filters(),
            OrExpression::OrExpression(vec) => vec.iter().flat_map(|e| e.filters()).collect(),
        }
    }
}
