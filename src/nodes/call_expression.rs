use crate::interpreter_visitor::{Execute, Interpreter};
use crate::nodes::AstExpression;
use crate::value::JsValue;

#[derive(Debug, Clone, PartialEq)]
pub struct CallExpressionNode {
    pub callee: Box<AstExpression>,
    pub params: Vec<AstExpression>,
}

impl Execute for CallExpressionNode {
    fn execute(&self, interpreter: &Interpreter) -> Result<JsValue, String> {
        interpreter.call_function(self.callee.as_ref(), self.params.as_ref(), false)
    }
}