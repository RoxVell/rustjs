use crate::nodes::AstExpression;

#[derive(Debug, Clone, PartialEq)]
pub struct ObjectPropertyNode {
    pub computed: bool,
    pub key: Box<AstExpression>,
    pub value: Box<AstExpression>,
}
