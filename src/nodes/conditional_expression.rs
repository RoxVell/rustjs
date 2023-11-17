use crate::interpreter::ast_interpreter::{Interpreter};
use crate::nodes::{AstExpression, Execute};
use crate::value::JsValue;

#[derive(Debug, Clone, PartialEq)]
pub struct ConditionalExpressionNode {
    pub test: Box<AstExpression>,
    pub consequent: Box<AstExpression>,
    pub alternative: Box<AstExpression>,
}

impl Execute for ConditionalExpressionNode {
    fn execute(&self, interpreter: &Interpreter) -> Result<JsValue, String> {
        let test = self.test.execute(interpreter)?;

        let branch = if test.to_bool() {
            &self.consequent
        } else {
            &self.alternative
        };

        return branch.execute(interpreter);
    }
}
