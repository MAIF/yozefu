use crate::search::{
    atom::Atom,
    compare::{CompareExpression, NumberOperator, StringOperator},
    expression::{parse_or_expression, AndExpression, Expression},
    term::Term,
};

#[test]
fn test_parse_term() {
    assert_eq!(
        parse_or_expression("!offset == 8"),
        Ok((
            "",
            Expression::OrTerm(AndExpression::AndTerm(Term::Not(Atom::Compare(
                CompareExpression::Offset(NumberOperator::Equal, 8)
            ))))
        ))
    )
}

#[test]
fn test_parse_and_expression() {
    assert_eq!(
        parse_or_expression("offset == 0 && topic == 'boite'"),
        Ok((
            "",
            Expression::OrTerm(AndExpression::AndExpression(vec!(
                Term::Atom(Atom::Compare(CompareExpression::Offset(
                    NumberOperator::Equal,
                    0
                ))),
                Term::Atom(Atom::Compare(CompareExpression::Topic(
                    StringOperator::Equal,
                    "boite".to_string()
                ))),
            )))
        ))
    )
}
