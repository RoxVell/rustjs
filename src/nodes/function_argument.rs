use crate::nodes::AstExpression;
use crate::nodes::identifier::IdentifierNode;

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionArgument {
    pub name: IdentifierNode,
    pub default_value: Option<Box<AstExpression>>,
}
