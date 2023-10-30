use crate::nodes::{AstExpression, AstStatement, Execute, Interpreter, JsValue};
use crate::scanner::Token;

#[derive(Debug, Clone, PartialEq)]
pub struct NumberLiteralNode {
    pub value: f64,
    pub token: Token,
}

impl Execute for NumberLiteralNode {
    fn execute(&self, _: &Interpreter) -> Result<JsValue, String> {
        Ok(JsValue::Number(self.value))
    }
}

// impl Into<AstExpression> for NumberLiteralNode {
//     fn into(self) -> AstExpression {
//         AstExpression::NumberLiteral(self)
//     }
// }

impl Into<AstStatement> for NumberLiteralNode {
    fn into(self) -> AstStatement {
        AstExpression::NumberLiteral(self).into()
    }
}
