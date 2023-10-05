use std::fmt::{Debug, Formatter};
use crate::interpreter::ast_interpreter::{Execute, Interpreter};
use crate::scanner::Token;
use crate::value::JsValue;

#[derive(Clone, PartialEq)]
pub struct StringLiteralNode {
    pub value: String,
    pub token: Token,
}

impl Execute for StringLiteralNode {
    fn execute(&self, _: &Interpreter) -> Result<JsValue, String> {
        Ok(JsValue::String(self.value.clone()))
    }
}

impl Debug for StringLiteralNode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "\"{}\"", self.value)
    }
}
