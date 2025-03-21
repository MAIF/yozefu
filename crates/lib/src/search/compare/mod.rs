#[cfg(feature = "native")]
pub mod equal;
#[cfg(feature = "native")]
pub mod expression;
#[cfg(feature = "native")]
pub mod number;
pub mod string;
#[cfg(feature = "native")]
pub use equal::parse_equal;
#[cfg(feature = "native")]
pub use expression::CompareExpression;
#[cfg(feature = "native")]
pub use expression::parse_compare;
#[cfg(feature = "native")]
pub use number::NumberOperator;
pub use string::StringOperator;

#[cfg(test)]
pub mod mod_test;
