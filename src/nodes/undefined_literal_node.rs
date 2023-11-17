use crate::nodes::{Execute, Interpreter, JsValue, Token};

#[derive(Debug, Clone, PartialEq)]
pub struct UndefinedLiteralNode(pub Token);

impl Execute for UndefinedLiteralNode {
    fn execute(&self, _: &Interpreter) -> Result<JsValue, String> {
        Ok(JsValue::Undefined)
    }
}
