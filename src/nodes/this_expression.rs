use crate::interpreter_visitor::Interpreter;
use crate::nodes::{Execute, JsValue, Token};

#[derive(Debug, Clone, PartialEq)]
pub struct ThisExpressionNode {
    pub token: Token,
}

impl Execute for ThisExpressionNode {
    fn execute(&self, interpreter: &Interpreter) -> Result<JsValue, String> {
        return Ok(interpreter.environment.borrow().borrow().get_context());
    }
}