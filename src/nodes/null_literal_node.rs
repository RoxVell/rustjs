use crate::nodes::{Execute, Interpreter, JsValue, Token};

#[derive(Debug, Clone, PartialEq)]
pub struct NullLiteralNode(pub Token);

impl Execute for NullLiteralNode {
    fn execute(&self, _: &Interpreter) -> Result<JsValue, String> {
        Ok(JsValue::Null)
    }
}
