use crate::interpreter::ast_interpreter::{Execute, Interpreter};
use crate::nodes::AstStatement;
use crate::value::JsValue;

#[derive(Debug, Clone, PartialEq)]
pub struct ProgramNode {
    pub statements: Vec<AstStatement>,
}

impl Execute for ProgramNode {
    fn execute(&self, interpreter: &Interpreter) -> Result<JsValue, String> {
        self.statements.execute(interpreter)
    }
}
