use std::fmt::{Debug, Formatter};
use crate::interpreter_visitor::{Execute, Interpreter};
use crate::node::GetSpan;
use crate::nodes::{AstExpression, AstStatement};
use crate::scanner::{TextSpan, Token};
use crate::value::JsValue;

#[derive(Clone, PartialEq)]
pub struct IdentifierNode {
    pub id: String,
    pub token: Token,
}

impl Execute for IdentifierNode {
    fn execute(&self, interpreter: &Interpreter) -> Result<JsValue, String> {
        Ok(interpreter
            .environment
            .borrow()
            .borrow()
            .get_variable_value(&self.id))
    }
}

impl Debug for IdentifierNode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Identifier(\"{}\")", self.id)
    }
}

impl GetSpan for IdentifierNode {
    fn get_span(&self) -> TextSpan {
        self.token.span.clone()
    }
}

impl Into<AstExpression> for IdentifierNode {
    fn into(self) -> AstExpression {
        AstExpression::Identifier(self)
    }
}

impl From<IdentifierNode> for AstStatement {
    fn from(value: IdentifierNode) -> Self {
        AstExpression::Identifier(value).into()
    }
}