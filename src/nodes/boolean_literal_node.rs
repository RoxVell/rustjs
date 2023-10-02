use crate::interpreter_visitor::{Execute, Interpreter};
use crate::scanner::Token;
use crate::value::JsValue;

#[derive(Debug, Clone, PartialEq)]
pub struct BooleanLiteralNode {
    pub value: bool,
    pub token: Token,
}

impl Execute for BooleanLiteralNode {
    fn execute(&self, _: &Interpreter) -> Result<JsValue, String> {
        Ok(JsValue::Boolean(self.value))
    }
}
