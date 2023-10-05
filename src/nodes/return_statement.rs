use crate::interpreter::ast_interpreter::{Execute, Interpreter};
use crate::nodes::AstExpression;
use crate::value::JsValue;

#[derive(Debug, Clone, PartialEq)]
pub struct ReturnStatementNode {
    pub expression: Box<AstExpression>,
}

impl Execute for ReturnStatementNode {
    fn execute(&self, interpreter: &Interpreter) -> Result<JsValue, String> {
        self.expression.execute(interpreter)
    }
}
