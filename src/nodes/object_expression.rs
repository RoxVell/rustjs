use crate::interpreter::ast_interpreter::{Interpreter};
use crate::nodes::Execute;
use crate::nodes::object_property::ObjectPropertyNode;
use crate::value::JsValue;
use crate::value::object::JsObject;

#[derive(Debug, Clone, PartialEq)]
pub struct ObjectExpressionNode {
    pub properties: Vec<ObjectPropertyNode>,
}

impl Execute for ObjectExpressionNode {
    fn execute(&self, interpreter: &Interpreter) -> Result<JsValue, String> {
        let mut object_value = JsObject::empty();

        for property in &self.properties {
            let key = interpreter.eval_member_expression_key(&property.key, property.computed)?;
            object_value.add_property(&key, property.value.execute(interpreter)?);
        }

        return Ok(object_value.into());
    }
}
