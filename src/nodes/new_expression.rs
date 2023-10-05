use crate::interpreter::ast_interpreter::{Execute, Interpreter};
use crate::nodes::AstExpression;
use crate::value::JsValue;

#[derive(Debug, Clone, PartialEq)]
pub struct NewExpressionNode {
    pub callee: Box<AstExpression>,
    pub arguments: Vec<AstExpression>,
}

impl Execute for NewExpressionNode {
    fn execute(&self, interpreter: &Interpreter) -> Result<JsValue, String> {
        let callee = self.callee.execute(interpreter)?;

        if !callee.is_function() {
            return Err("Cannot call non-function value".to_string());
        }

        interpreter.call_function(self.callee.as_ref(), self.arguments.as_ref(), true)
    }
}