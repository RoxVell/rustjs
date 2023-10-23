use crate::interpreter::ast_interpreter::{Execute, Interpreter};
use crate::nodes::AstExpression;
use crate::value::JsValue;

#[derive(Debug, Clone, PartialEq)]
pub struct MemberExpressionNode {
    pub computed: bool,
    pub object: Box<AstExpression>,
    pub property: Box<AstExpression>,
}

impl Execute for MemberExpressionNode {
    fn execute(&self, interpreter: &Interpreter) -> Result<JsValue, String> {
        let property_key = interpreter.eval_member_expression_key(&self.property, self.computed)?;
        let resolved_object = self.object.execute(interpreter)?;

        match resolved_object {
            JsValue::Object(object) => {
                Ok(object.borrow_mut().get_property_value(property_key.as_str()))
            },
            _ => Err("Is not an object".to_string())
        }
    }
}
