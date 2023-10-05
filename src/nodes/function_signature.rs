use crate::nodes::AstStatement;
use crate::nodes::function_argument::FunctionArgument;
use crate::nodes::identifier::IdentifierNode;

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionSignature {
    pub name: Box<IdentifierNode>,
    pub arguments: Vec<FunctionArgument>,
    pub body: Box<AstStatement>,
}
