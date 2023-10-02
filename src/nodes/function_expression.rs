use crate::interpreter_visitor::{Execute, Interpreter};
use crate::nodes::{AstStatement, FunctionArgument};
use crate::value::JsValue;

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionExpressionNode {
    pub arguments: Vec<FunctionArgument>,
    pub body: Box<AstStatement>,
}

impl Execute for FunctionExpressionNode {
    fn execute(&self, interpreter: &Interpreter) -> Result<JsValue, String> {
        let function = interpreter.create_js_function(&self.arguments, *self.body.clone());
        let mut object = function.to_object();
        object.add_property("prototype", JsValue::object([]));
        // object.set_prototype(JsObject::empty_ref());
        return Ok(object.to_js_value());
    }
}
