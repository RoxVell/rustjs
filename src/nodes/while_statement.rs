use crate::interpreter_visitor::{Execute, Interpreter};
use crate::nodes::{AstExpression, AstStatement};
use crate::value::JsValue;

#[derive(Debug, Clone, PartialEq)]
pub struct WhileStatementNode {
    pub condition: Box<AstExpression>,
    pub body: Box<AstStatement>,
}

impl Execute for WhileStatementNode {
    fn execute(&self, interpreter: &Interpreter) -> Result<JsValue, String> {
        while self.condition.execute(interpreter).unwrap().to_bool() {
            self.body.execute(interpreter).unwrap();
        }

        Ok(JsValue::Undefined)
    }
}
