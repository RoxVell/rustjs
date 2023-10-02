use crate::interpreter_visitor::Interpreter;
use crate::nodes::{AstExpression, AstStatement, Execute, JsValue};

#[derive(Debug, Clone, PartialEq)]
pub struct IfStatementNode {
    pub condition: Box<AstExpression>,
    pub then_branch: Box<AstStatement>,
    pub else_branch: Option<Box<AstStatement>>,
}

impl Execute for IfStatementNode {
    fn execute(&self, interpreter: &Interpreter) -> Result<JsValue, String> {
        let condition_value = self.condition.execute(interpreter)?;

        if condition_value.to_bool() {
            self.then_branch.execute(interpreter)?;
        } else if let Some(node) = self.else_branch.as_ref() {
            node.execute(interpreter)?;
        }

        return Ok(JsValue::Undefined);
    }
}
