mod string_literal_node;

use enum_dispatch::enum_dispatch;
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
mod null_literal_node;
mod undefined_literal_node;
mod template_string_literal;
mod unary_expression;
mod break_statement;

pub use object_property::*;
pub use function_signature::*;
pub use class_declaration::*;
pub use number_literal_node::NumberLiteralNode;
pub use crate::interpreter::ast_interpreter::Interpreter;
pub use crate::node::GetSpan;
pub use crate::nodes::block_statement::BlockStatementNode;
pub use crate::nodes::for_statement::ForStatementNode;
pub use crate::nodes::identifier::IdentifierNode;
pub use crate::nodes::program::ProgramNode;
pub use crate::nodes::return_statement::ReturnStatementNode;
pub use crate::nodes::variable_declaration::{VariableDeclarationKind, VariableDeclarationNode};
pub use crate::nodes::while_statement::WhileStatementNode;
pub use crate::scanner::{TextSpan, Token};
pub use crate::value::JsValue;
pub use function_argument::FunctionArgument;
pub use null_literal_node::NullLiteralNode;
pub use undefined_literal_node::UndefinedLiteralNode;
pub use crate::nodes::array_expression::ArrayExpressionNode;
pub use crate::nodes::assignment_expression::{AssignmentExpressionNode, AssignmentOperator};
pub use crate::nodes::binary_expression::{BinaryExpressionNode, BinaryOperator};
pub use crate::nodes::break_statement::BreakStatementNode;
pub use crate::nodes::call_expression::CallExpressionNode;
pub use crate::nodes::conditional_expression::ConditionalExpressionNode;
pub use crate::nodes::function_declaration::FunctionDeclarationNode;
pub use crate::nodes::function_expression::FunctionExpressionNode;
pub use crate::nodes::member_expression::MemberExpressionNode;
pub use crate::nodes::new_expression::NewExpressionNode;
pub use crate::nodes::object_expression::ObjectExpressionNode;
pub use crate::nodes::template_string_literal::{TemplateStringLiteralNode, TemplateElement};
pub use crate::nodes::this_expression::ThisExpressionNode;
pub use crate::nodes::unary_expression::{UnaryExpressionNode, UnaryOperator};

#[derive(Debug, Clone, PartialEq)]
#[enum_dispatch(Execute)]
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
    BreakStatement(BreakStatementNode),
}

impl AsRef<AstStatement> for AstStatement {
    fn as_ref(&self) -> &AstStatement {
        &self
    }
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

#[derive(Debug, Clone, PartialEq)]
#[enum_dispatch(Execute)]
pub enum AstExpression {
    StringLiteral(StringLiteralNode),
    TemplateStringLiteral(TemplateStringLiteralNode),
    NumberLiteral(NumberLiteralNode),
    BooleanLiteral(BooleanLiteralNode),
    NullLiteral(NullLiteralNode),
    UndefinedLiteral(UndefinedLiteralNode),
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
    UnaryExpression(UnaryExpressionNode),
}

impl GetSpan for AstExpression {
    fn get_span(&self) -> TextSpan {
        match self {
            AstExpression::StringLiteral(node) => node.token.span.clone(),
            AstExpression::NumberLiteral(node) => node.token.span.clone(),
            AstExpression::BooleanLiteral(node) => node.token.span.clone(),
            AstExpression::NullLiteral(node) => node.0.span.clone(),
            AstExpression::UndefinedLiteral(node) => node.0.span.clone(),
            AstExpression::Identifier(node) => node.token.span.clone(),
            AstExpression::MemberExpression(node) => TextSpan::new(node.object.get_span().start, node.property.get_span().end),
            AstExpression::BinaryExpression(node) => TextSpan::new(node.left.get_span().start, node.right.get_span().end),
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

#[enum_dispatch]
pub trait Execute {
    fn execute(&self, interpreter: &Interpreter) -> Result<JsValue, String>;
}

#[enum_dispatch]
pub trait IsSimple {
    /// checks whether a node is "simple", meaning it can be computed at the compile stage
    fn is_simple(&self) -> bool;
}

impl IsSimple for AstStatement {
    fn is_simple(&self) -> bool {
        match self {
            AstStatement::ProgramStatement(node) => node.statements.iter().all(|x| x.is_simple()),
            AstStatement::BlockStatement(node) => node.statements.iter().all(|x| x.is_simple()),
            AstStatement::ExpressionStatement(node) => node.is_simple(),
            AstStatement::IfStatement(node) => {
                if node.condition.is_simple() {
                    node.then_branch.is_simple()
                } else {
                    node.else_branch.is_some() && node.else_branch.as_ref().unwrap().is_simple()
                }
            }
            AstStatement::BreakStatement(_)
                | AstStatement::ForStatement(_)
                | AstStatement::WhileStatement(_)
                | AstStatement::FunctionDeclaration(_)
                | AstStatement::ReturnStatement(_)
                | AstStatement::VariableDeclaration(_) => false,
        }
    }
}

impl IsSimple for AstExpression {
    fn is_simple(&self) -> bool {
        match self {
            AstExpression::StringLiteral(_)
                | AstExpression::NumberLiteral(_)
                | AstExpression::NullLiteral(_)
                | AstExpression::UndefinedLiteral(_)
                | AstExpression::BooleanLiteral(_) => true,
            AstExpression::ThisExpression(_)
                | AstExpression::Identifier(_)
                | AstExpression::AssignmentExpression(_)
                | AstExpression::CallExpression(_)
                | AstExpression::MemberExpression(_)
                | AstExpression::NewExpression(_)
                | AstExpression::ObjectExpression(_)
                | AstExpression::ClassDeclaration(_)
                | AstExpression::ArrayExpression(_)
                | AstExpression::FunctionExpression(_) => false,
            AstExpression::TemplateStringLiteral(node) => node.elements.iter().all(|x| x.is_simple()),
            AstExpression::BinaryExpression(node) => node.left.is_simple() && node.right.is_simple(),
            AstExpression::ConditionalExpression(node) => if node.test.is_simple() { node.consequent.is_simple() } else { node.alternative.is_simple() },
            AstExpression::UnaryExpression(node) => node.expression.is_simple(),
        }
    }
}