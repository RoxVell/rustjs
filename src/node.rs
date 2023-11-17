use crate::scanner::TextSpan;
use crate::keywords::{CONST_KEYWORD, FALSE_KEYWORD, LET_KEYWORD, TRUE_KEYWORD};
use crate::nodes::*;
use crate::visitor::Visitor;

pub trait GetSpan {
    fn get_span(&self) -> TextSpan;
}

pub struct Printer {
    ident: u32,
    level: u32,
    pub(crate) result: String,
}

impl Printer {
    pub fn new(ident: u32) -> Self {
        Self {
            ident,
            level: 0,
            result: String::new(),
        }
    }
}

impl Visitor for Printer {
    fn visit_block_statement(&mut self, stmt: &BlockStatementNode) {
        self.result += "{\n";
        self.level += 1;
        stmt.statements.iter().for_each(|stmt| {
            let spaces = " ".repeat((self.ident * self.level) as usize);
            self.result += spaces.as_str();
            self.visit_statement(stmt)
        });
        self.result += "}";
    }

    fn visit_if_statement(&mut self, stmt: &IfStatementNode) {
        self.result += "if (";
        self.visit_expression(&stmt.condition);
        self.result += ") ";

        self.visit_statement(&stmt.then_branch);

        if let Some(else_branch) = &stmt.else_branch {
            self.result += " else ";
            self.visit_statement(else_branch);
        }
    }

    fn visit_expression_statement(&mut self, stmt: &AstExpression) {
        println!("visit_expression_statement {stmt:?}");
        self.visit_expression(stmt);
        self.result += ";\n";
    }

    fn visit_string_literal(&mut self, stmt: &StringLiteralNode) {
        self.result += stmt.value.as_str();
        // println!("visit_string_literal: {}", stmt.value);
    }

    fn visit_number_literal(&mut self, stmt: &NumberLiteralNode) {
        self.result += stmt.value.to_string().as_str();
        // println!("visit_number_literal: {}", stmt.value);
    }

    fn visit_binary_expression(&mut self, stmt: &BinaryExpressionNode) {
        println!("visit_binary_expression");
        self.visit_expression(stmt.left.as_ref());
        self.result += " ";
        self.result += match stmt.operator {
            BinaryOperator::Add => "+",
            BinaryOperator::Sub => "-",
            BinaryOperator::Div => "/",
            BinaryOperator::Mul => "*",
            BinaryOperator::LogicalOr => "||",
            BinaryOperator::LogicalAnd => "&&",
            BinaryOperator::MoreThan => ">",
            BinaryOperator::MoreThanOrEqual => ">=",
            BinaryOperator::LessThan => "<",
            BinaryOperator::LessThanOrEqual => "<=",
            BinaryOperator::Equality => "==",
            BinaryOperator::Inequality => "!=",
            BinaryOperator::MulMul => "**",
        };
        self.result += " ";

        self.visit_expression(stmt.right.as_ref());
    }

    fn visit_boolean_literal(&mut self, stmt: &BooleanLiteralNode) {
        self.result += if stmt.value { TRUE_KEYWORD } else { FALSE_KEYWORD };
    }

    fn visit_program_statement(&mut self, stmt: &ProgramNode) {
        stmt.statements.iter().for_each(|stmt| {
            let spaces = " ".repeat((self.ident * self.level) as usize);
            self.result += spaces.as_str();
            self.visit_statement(stmt)
        });
    }

    fn visit_variable_declaration(&mut self, stmt: &VariableDeclarationNode) {
        self.result += match stmt.kind {
            VariableDeclarationKind::Let => LET_KEYWORD,
            VariableDeclarationKind::Const => CONST_KEYWORD
        };

        self.result += " ";

        self.visit_identifier_node(&stmt.id);

        self.result += " = ";

        if stmt.value.is_some() {
            self.visit_expression(stmt.value.as_ref().unwrap());
        }

        self.result += ";\n";
    }

    fn visit_identifier_node(&mut self, stmt: &IdentifierNode) {
        self.result += stmt.id.as_str();
        // println!("visit_identifier_declaration {}", stmt.id);
    }
}