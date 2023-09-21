use crate::node::*;
use crate::scanner::Token;

pub trait Visitor {
    fn visit_statement(&mut self, stmt: &AstStatement) {
        match stmt {
            AstStatement::ProgramStatement(stmt) => self.visit_program_statement(stmt),
            AstStatement::VariableDeclaration(stmt) => self.visit_variable_declaration(stmt),
            AstStatement::BlockStatement(stmt) => self.visit_block_statement(stmt),
            AstStatement::WhileStatement(node) => self.visit_while_statement(node),
            AstStatement::ForStatement(stmt) => self.visit_for_statement(stmt),
            AstStatement::FunctionDeclaration(stmt) => self.visit_function_declaration(stmt),
            AstStatement::ReturnStatement(node) => self.visit_return_statement(node),
            AstStatement::ExpressionStatement(stmt) => self.visit_expression_statement(stmt),
            AstStatement::IfStatement(stmt) => self.visit_if_statement(stmt),
            AstStatement::BreakStatement(token) => self.visit_break_statement(token),
        }
    }

    fn visit_break_statement(&mut self, _: &Token) {}

    fn visit_while_statement(&mut self, node: &WhileStatementNode) {
        self.visit_expression(&node.condition);
        self.visit_statement(&node.body);
    }

    fn visit_return_statement(&mut self, node: &ReturnStatementNode) {
        self.visit_expression(&node.expression);
    }

    fn visit_for_statement(&mut self, stmt: &ForStatementNode) {
        if let Some(init) = &stmt.init {
            self.visit_statement(init);
        }

        if let Some(test) = &stmt.test {
            self.visit_expression(test);
        }

        if let Some(update) = &stmt.update {
            self.visit_expression(update);
        }

        self.visit_statement(&stmt.body);
    }

    fn visit_class_declaration(&mut self, stmt: &ClassDeclarationNode) {
        self.visit_identifier_node(stmt.name.as_ref());
        if let Some(parent) = &stmt.parent {
            self.visit_identifier_node(parent);
        }
        stmt.methods.iter().for_each(|x| self.visit_class_method(x));
    }

    fn visit_class_method(&mut self, stmt: &ClassMethodNode) {
        self.visit_function_signature(&stmt.function_signature);
    }

    fn visit_function_declaration(&mut self, stmt: &FunctionDeclarationNode) {
        self.visit_function_signature(&stmt.function_signature);
    }

    fn visit_function_signature(&mut self, stmt: &FunctionSignature) {
        self.visit_identifier_node(&stmt.name);
        stmt.arguments.iter().for_each(|x| self.visit_function_argument(x));
        self.visit_statement(&stmt.body);
    }

    fn visit_function_argument(&mut self, stmt: &FunctionArgument) {
        self.visit_identifier_node(&stmt.name);
        if let Some(value) = &stmt.default_value {
            self.visit_expression(value);
        }
    }

    fn visit_block_statement(&mut self, stmt: &BlockStatementNode) {
        stmt.statements.iter().for_each(|stmt| self.visit_statement(stmt));
    }

    fn visit_if_statement(&mut self, stmt: &IfStatementNode) {
        self.visit_expression(&stmt.condition);

        self.visit_statement(&stmt.then_branch);

        if let Some(else_branch) = &stmt.else_branch {
            self.visit_statement(else_branch);
        }
    }

    fn visit_expression_statement(&mut self, stmt: &AstExpression) {
        self.visit_expression(stmt);
    }

    fn visit_string_literal(&mut self, _: &StringLiteralNode) {}

    fn visit_number_literal(&mut self, _: &NumberLiteralNode) {}

    fn visit_expression(&mut self, stmt: &AstExpression) {
        match stmt {
            AstExpression::StringLiteral(node) => self.visit_string_literal(node),
            AstExpression::NumberLiteral(node) => self.visit_number_literal(node),
            AstExpression::BooleanLiteral(node) => self.visit_boolean_literal(node),
            AstExpression::NullLiteral(_) => self.visit_null_literal(),
            AstExpression::UndefinedLiteral(_) => self.visit_undefined_literal(),
            AstExpression::ThisExpression(token) => self.visit_this_expression(token),
            AstExpression::Identifier(node) => self.visit_identifier_node(node),
            AstExpression::BinaryExpression(node) => self.visit_binary_expression(node),
            AstExpression::AssignmentExpression(node) => self.visit_assignment_expression(node),
            AstExpression::FunctionExpression(node) => self.visit_function_expression(node),
            AstExpression::CallExpression(node) => self.visit_call_expression(node),
            AstExpression::ConditionalExpression(node) => self.visit_conditional_expression(node),
            AstExpression::MemberExpression(node) => self.visit_member_expression(node),
            AstExpression::NewExpression(node) => self.visit_new_expression(node),
            AstExpression::ObjectExpression(node) => self.visit_object_expression(node),
            AstExpression::ClassDeclaration(node) => self.visit_class_declaration(node),
            AstExpression::ArrayExpression(node) => self.visit_array_expression(node),
        }
    }

    fn visit_conditional_expression(&mut self, node: &ConditionalExpressionNode) {
        self.visit_expression(&node.test);
        self.visit_expression(&node.consequent);
        self.visit_expression(&node.alternative);
    }

    fn visit_array_expression(&mut self, node: &ArrayExpressionNode) {
        node.items.iter().for_each(|x| self.visit_expression(x));
    }

    fn visit_function_expression(&mut self, node: &FunctionExpressionNode) {
        node.arguments.iter().for_each(|x| self.visit_function_argument(x));
        self.visit_statement(&node.body);
    }

    fn visit_undefined_literal(&mut self) {}

    fn visit_null_literal(&mut self) {}

    fn visit_this_expression(&mut self, _: &Token) {}

    fn visit_object_expression(&mut self, node: &ObjectExpressionNode) {
        node.properties.iter().for_each(|x| self.visit_object_property(x));
    }

    fn visit_object_property(&mut self, node: &ObjectPropertyNode) {
        self.visit_expression(&node.value);
        self.visit_expression(&node.key);
    }

    fn visit_member_expression(&mut self, stmt: &MemberExpressionNode) {
        self.visit_expression(&stmt.object);
        self.visit_expression(&stmt.property);
    }

    fn visit_new_expression(&mut self, stmt: &NewExpressionNode) {
        self.visit_expression(&stmt.callee);
        stmt.arguments.iter().for_each(|x| self.visit_expression(x));
    }

    fn visit_call_expression(&mut self, stmt: &CallExpressionNode) {
        self.visit_expression(&stmt.callee);
        stmt.params.iter().for_each(|x| self.visit_expression(x));
    }

    fn visit_assignment_expression(&mut self, stmt: &AssignmentExpressionNode) {
        self.visit_expression(&stmt.left);
        self.visit_expression(&stmt.right);
    }

    fn visit_binary_expression(&mut self, stmt: &BinaryExpressionNode) {
        self.visit_expression(stmt.left.as_ref());
        self.visit_expression(stmt.right.as_ref());
    }

    fn visit_boolean_literal(&mut self, _: &BooleanLiteralNode) {}

    fn visit_program_statement(&mut self, stmt: &ProgramNode) {
        stmt.statements.iter().for_each(|stmt| self.visit_statement(stmt));
    }

    fn visit_variable_declaration(&mut self, _: &VariableDeclarationNode) {}

    fn visit_identifier_node(&mut self, _: &IdentifierNode) {}
}
