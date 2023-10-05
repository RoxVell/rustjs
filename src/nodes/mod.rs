mod string_literal_node;
pub use string_literal_node::StringLiteralNode;
mod boolean_literal_node;
pub use boolean_literal_node::BooleanLiteralNode;

mod if_statement;
pub use if_statement::IfStatementNode;

mod number_literal_node;
mod array_expression;
mod block_statement;
mod identifier;
mod binary_expression;
mod variable_declaration;
mod while_statement;
mod assignment_expression;
mod program;
mod for_statement;
mod call_expression;
mod member_expression;
mod conditional_expression;
mod return_statement;
mod function_declaration;
mod function_signature;
mod function_argument;
mod function_expression;
mod class_declaration;
mod object_property;
mod object_expression;
mod new_expression;
mod this_expression;

pub use object_property::*;
pub use function_signature::*;
pub use class_declaration::*;
pub use number_literal_node::NumberLiteralNode;
pub use crate::interpreter::ast_interpreter::{Execute, Interpreter};
pub use crate::node::GetSpan;
pub use crate::nodes::block_statement::BlockStatementNode;
pub use crate::nodes::for_statement::ForStatementNode;
pub use crate::nodes::identifier::IdentifierNode;
pub use crate::nodes::program::ProgramNode;
pub use crate::nodes::return_statement::ReturnStatementNode;
pub use crate::nodes::variable_declaration::{VariableDeclarationNode, VariableDeclarationKind};
pub use crate::nodes::while_statement::WhileStatementNode;
pub use crate::scanner::{TextSpan, Token};
pub use crate::value::JsValue;
pub use function_argument::FunctionArgument;
pub use crate::nodes::array_expression::ArrayExpressionNode;
pub use crate::nodes::assignment_expression::{AssignmentExpressionNode, AssignmentOperator};
pub use crate::nodes::binary_expression::{BinaryExpressionNode, BinaryOperator};
pub use crate::nodes::call_expression::CallExpressionNode;
pub use crate::nodes::conditional_expression::ConditionalExpressionNode;
pub use crate::nodes::function_declaration::FunctionDeclarationNode;
pub use crate::nodes::function_expression::FunctionExpressionNode;
pub use crate::nodes::member_expression::MemberExpressionNode;
pub use crate::nodes::new_expression::NewExpressionNode;
pub use crate::nodes::object_expression::ObjectExpressionNode;
pub use crate::nodes::this_expression::ThisExpressionNode;

#[derive(Debug, Clone, PartialEq)]
pub enum AstStatement {
    ProgramStatement(ProgramNode),
    VariableDeclaration(VariableDeclarationNode),
    BlockStatement(BlockStatementNode),
    WhileStatement(WhileStatementNode),
    ForStatement(ForStatementNode),
    FunctionDeclaration(FunctionDeclarationNode),
    ReturnStatement(ReturnStatementNode),
    ExpressionStatement(AstExpression),
    IfStatement(IfStatementNode),
    BreakStatement(Token),
}

impl Execute for Vec<AstStatement> {
    fn execute(&self, interpreter: &Interpreter) -> Result<JsValue, String> {
        let mut result = JsValue::Undefined;

        for i in self {
            result = i.execute(interpreter)?;
        }

        Ok(result)
    }
}

impl Execute for AstStatement {
    fn execute(&self, interpreter: &Interpreter) -> Result<JsValue, String> {
        match self {
            AstStatement::ProgramStatement(node) => node.execute(interpreter),
            AstStatement::VariableDeclaration(node) => node.execute(interpreter),
            AstStatement::BlockStatement(node) => node.execute(interpreter),
            AstStatement::WhileStatement(node) => node.execute(interpreter),
            AstStatement::ForStatement(node) => node.execute(interpreter),
            AstStatement::FunctionDeclaration(node) => node.execute(interpreter),
            AstStatement::ReturnStatement(node) => node.execute(interpreter),
            AstStatement::ExpressionStatement(node) => node.execute(interpreter),
            AstStatement::IfStatement(node) => node.execute(interpreter),
            AstStatement::BreakStatement(_) => todo!(),
        }
    }
}

impl Into<AstStatement> for AstExpression {
    fn into(self) -> AstStatement {
        AstStatement::ExpressionStatement(self)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum AstExpression {
    StringLiteral(StringLiteralNode),
    NumberLiteral(NumberLiteralNode),
    BooleanLiteral(BooleanLiteralNode),
    NullLiteral(Token),
    UndefinedLiteral(Token),
    ThisExpression(ThisExpressionNode),
    Identifier(IdentifierNode),
    BinaryExpression(BinaryExpressionNode),
    AssignmentExpression(AssignmentExpressionNode),
    FunctionExpression(FunctionExpressionNode),
    CallExpression(CallExpressionNode),
    ConditionalExpression(ConditionalExpressionNode),
    MemberExpression(MemberExpressionNode),
    NewExpression(NewExpressionNode),
    ObjectExpression(ObjectExpressionNode),
    ClassDeclaration(ClassDeclarationNode),
    ArrayExpression(ArrayExpressionNode),
}

impl Execute for AstExpression {
    fn execute(&self, interpreter: &Interpreter) -> Result<JsValue, String> {
        match self {
            AstExpression::StringLiteral(node) => node.execute(interpreter),
            AstExpression::NumberLiteral(node) => node.execute(interpreter),
            AstExpression::BooleanLiteral(node) => node.execute(interpreter),
            AstExpression::NullLiteral(_) => Ok(JsValue::Null),
            AstExpression::UndefinedLiteral(_) => Ok(JsValue::Undefined),
            AstExpression::ThisExpression(node) => node.execute(interpreter),
            AstExpression::Identifier(node) => node.execute(interpreter),
            AstExpression::BinaryExpression(node) => node.execute(interpreter),
            AstExpression::AssignmentExpression(node) => node.execute(interpreter),
            AstExpression::FunctionExpression(node) => node.execute(interpreter),
            AstExpression::CallExpression(node) => node.execute(interpreter),
            AstExpression::ConditionalExpression(node) => node.execute(interpreter),
            AstExpression::MemberExpression(node) => node.execute(interpreter),
            AstExpression::NewExpression(node) => node.execute(interpreter),
            AstExpression::ObjectExpression(node) => node.execute(interpreter),
            AstExpression::ClassDeclaration(node) => node.execute(interpreter),
            AstExpression::ArrayExpression(node) => node.execute(interpreter),
        }
    }
}

impl GetSpan for AstExpression {
    fn get_span(&self) -> TextSpan {
        match self {
            AstExpression::StringLiteral(node) => node.token.span.clone(),
            AstExpression::NumberLiteral(node) => node.token.span.clone(),
            AstExpression::BooleanLiteral(node) => node.token.span.clone(),
            AstExpression::NullLiteral(node) => node.span.clone(),
            AstExpression::UndefinedLiteral(node) => node.span.clone(),
            AstExpression::Identifier(node) => node.token.span.clone(),
            _ => todo!()
            // AstExpression::ThisExpression(_) => {}
            // AstExpression::BinaryExpression(_) => {}
            // AstExpression::AssignmentExpression(_) => {}
            // AstExpression::FunctionExpression(_) => {}
            // AstExpression::CallExpression(_) => {}
            // AstExpression::ConditionalExpression(_) => {}
            // AstExpression::MemberExpression(_) => {}
            // AstExpression::NewExpression(_) => {}
            // AstExpression::ObjectExpression(_) => {}
            // AstExpression::ClassDeclaration(_) => {}
            // AstExpression::ArrayExpression(_) => {}
        }
    }
}