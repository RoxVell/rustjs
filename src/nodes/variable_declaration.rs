use crate::interpreter::ast_interpreter::{Interpreter};
use crate::nodes::{AstExpression, Execute};
use crate::nodes::identifier::IdentifierNode;
use crate::value::JsValue;

#[derive(Debug, Clone, PartialEq)]
pub enum VariableDeclarationKind {
    Let,
    Const,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VariableDeclarationNode {
    pub kind: VariableDeclarationKind,
    pub id: IdentifierNode,
    pub value: Option<Box<AstExpression>>,
}

impl Execute for VariableDeclarationNode {
    fn execute(&self, interpreter: &Interpreter) -> Result<JsValue, String> {
        let value = if let Some(value) = &self.value {
            value.execute(interpreter)?
        } else {
            JsValue::Undefined
        };
        return interpreter.environment
            .borrow()
            .borrow_mut()
            .define_variable(self.id.id.clone(), value, matches!(&self.kind, VariableDeclarationKind::Const))
            .map(|_| JsValue::Undefined);
    }
}