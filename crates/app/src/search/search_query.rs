use lib::{
    SearchQuery,
    search::{filter::Filter, offset::FromOffset},
};

use super::{Search, SearchContext};

impl Search for SearchQuery {
    fn offset(&self) -> Option<FromOffset> {
        self.from.clone().or(self.expression.offset())
    }

    fn matches(&self, context: &SearchContext) -> bool {
        self.expression.matches(context)
    }

    fn filters(&self) -> Vec<Filter> {
        self.expression.filters()
    }
}
