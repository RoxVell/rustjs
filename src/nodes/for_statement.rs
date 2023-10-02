use crate::interpreter_visitor::{Execute, Interpreter};
use crate::nodes::{AstExpression, AstStatement};
use crate::value::JsValue;

#[derive(Debug, Clone, PartialEq)]
pub struct ForStatementNode {
    pub init: Option<Box<AstStatement>>,
    pub test: Option<Box<AstExpression>>,
    pub update: Option<Box<AstExpression>>,
    pub body: Box<AstStatement>,
}

impl Execute for ForStatementNode {
    fn execute(&self, interpreter: &Interpreter) -> Result<JsValue, String> {
        interpreter.set_environment(interpreter.create_new_environment());

        if let Some(init) = &self.init {
            init.execute(interpreter)?;
        }

        while self.test.as_ref().unwrap().execute(interpreter)?.to_bool()
        {
            self.body.execute(interpreter)?;
            self.update.as_ref().unwrap().execute(interpreter)?;
        }

        interpreter.pop_environment();

        Ok(JsValue::Undefined)
    }
}
