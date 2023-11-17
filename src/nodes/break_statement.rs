use crate::nodes::{Execute, Interpreter, JsValue, Token};

#[derive(Debug, Clone, PartialEq)]
pub struct BreakStatementNode(pub Token);

impl Execute for BreakStatementNode {
    fn execute(&self, _: &Interpreter) -> Result<JsValue, String> {
        todo!()
    }
}
