use crate::interpreter::ast_interpreter::{Interpreter};
use crate::nodes::{AstStatement, Execute};
use crate::value::JsValue;

#[derive(Debug, Clone, PartialEq)]
pub struct BlockStatementNode {
    pub statements: Vec<AstStatement>,
}

impl Execute for BlockStatementNode {
    fn execute(&self, interpreter: &Interpreter) -> Result<JsValue, String> {
        let env = interpreter.create_new_environment();
        interpreter.set_environment(env);
        let result = self.statements.execute(interpreter);
        interpreter.pop_environment();
        return result;
    }
}
