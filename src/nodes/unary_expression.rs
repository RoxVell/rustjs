use crate::nodes::{AstExpression, Execute, Interpreter, JsValue};

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOperator {
    Plus,
    Minus,
    LogicalNot,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnaryExpressionNode {
    pub operator: UnaryOperator,
    pub expression: Box<AstExpression>,
}

impl Execute for UnaryExpressionNode {
    fn execute(&self, interpreter: &Interpreter) -> Result<JsValue, String> {
        let value: JsValue = self.expression.execute(interpreter)?;

        match self.operator {
            UnaryOperator::Plus => Ok(value.unary_plus()),
            UnaryOperator::Minus => Ok(value.unary_minus()),
            UnaryOperator::LogicalNot => Ok(value.unary_logical_not())
        }
    }
}
