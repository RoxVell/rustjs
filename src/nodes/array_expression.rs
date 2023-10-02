use crate::interpreter_visitor::{Execute, Interpreter};
use crate::nodes::AstExpression;
use crate::value::JsValue;
use crate::value::object::JsObject;

#[derive(Debug, Clone, PartialEq)]
pub struct ArrayExpressionNode {
    pub items: Vec<AstExpression>,
}

impl Execute for ArrayExpressionNode {
    fn execute(&self, interpreter: &Interpreter) -> Result<JsValue, String> {
        let array_items: Vec<JsValue> = self.items.iter().map(|x| x.execute(interpreter).unwrap()).collect();
        return Ok(JsObject::array(array_items).to_js_value());
    }
}