use crate::interpreter_visitor::{Execute, Interpreter};
use crate::nodes::function_signature::FunctionSignature;
use crate::value::JsValue;
use crate::value::object::JsObject;

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDeclarationNode {
    pub function_signature: FunctionSignature
}

impl Execute for FunctionDeclarationNode {
    fn execute(&self, interpreter: &Interpreter) -> Result<JsValue, String> {
        let js_function_value: JsValue = interpreter.create_js_function(&self.function_signature.arguments, *self.function_signature.body.clone()).into();

        if let JsValue::Object(function) = &js_function_value {
            function.borrow_mut().set_prototype(JsObject::empty_ref());
        }

        interpreter.environment.borrow()
            .borrow_mut()
            .define_variable(self.function_signature.name.id.clone(), js_function_value.clone().into(), false)?;
        return Ok(js_function_value);
    }
}