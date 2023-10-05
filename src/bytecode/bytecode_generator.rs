use crate::bytecode::opcode::OpCode;
use crate::nodes::*;
use crate::value::JsValue;
use crate::visitor::Visitor;

#[derive(Debug)]
pub struct CodeBlock {
    pub bytecode: Vec<u8>,
    pub constants: Vec<JsValue>,
}

impl CodeBlock {
    pub fn new() -> Self {
        Self {
            bytecode: vec![],
            constants: vec![],
        }
    }
}

struct BytecodeCompiler {
    code_block: CodeBlock
}

impl BytecodeCompiler {
    fn new() -> Self {
        Self {
            code_block: CodeBlock::new(),
        }
    }

    pub fn compile(&mut self, ast: &AstStatement) {
        self.code_block = CodeBlock::new();
        self.visit_statement(ast);
        // self.emit_opcode(OpCode::Halt);
    }

    fn add_constant(&mut self, value: JsValue) -> usize {
        let result = self.code_block.constants.iter().position(|x| *x == value);

        match result {
            None => {
                self.code_block.constants.push(value);
                self.code_block.constants.len() - 1
            }
            Some(idx) => idx
        }
    }

    fn emit_opcode(&mut self, opcode: OpCode) {
        // println!("emit_opcode {opcode:?}");
        self.code_block.bytecode.push(opcode as u8);
    }

    fn emit(&mut self, opcode: OpCode, operands: &[u8]) {
        self.emit_opcode(opcode);
        self.emit_operands(operands);
    }

    fn emit_operands(&mut self, operands: &[u8]) {
        for operand in operands {
            self.code_block.bytecode.push(*operand);
        }
    }

    fn offset(&self) -> usize {
        self.code_block.bytecode.len()
    }

    fn patch_jump_address(&mut self, src: usize, dest: usize) {
        self.code_block.bytecode[src] = dest as u8;
    }
}

impl Visitor for BytecodeCompiler {
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

        self.emit(OpCode::JumpIfFalse, &[0]);
        let else_branch_address = self.offset() - 1;

        self.visit_statement(&stmt.then_branch);
        self.emit(OpCode::Jump, &[0]);
        let then_branch_end = self.offset() - 1;
        self.patch_jump_address(else_branch_address, self.offset());

        if let Some(else_branch) = &stmt.else_branch {
            self.visit_statement(else_branch);
        }

        self.patch_jump_address(then_branch_end, self.offset());
    }

    fn visit_expression_statement(&mut self, stmt: &AstExpression) {
        self.visit_expression(stmt);
    }

    fn visit_string_literal(&mut self, node: &StringLiteralNode) {
        let index = self.add_constant(JsValue::String(node.value.clone())) as u8;
        self.emit(OpCode::PushLiteral, &index.to_ne_bytes());
    }

    fn visit_number_literal(&mut self, node: &NumberLiteralNode) {
        let index = self.add_constant(JsValue::Number(node.value)) as u8;
        self.emit(OpCode::PushLiteral, &index.to_ne_bytes());
    }

    fn visit_expression(&mut self, stmt: &AstExpression) {
        match stmt {
            AstExpression::StringLiteral(node) => self.visit_string_literal(node),
            AstExpression::NumberLiteral(node) => self.visit_number_literal(node),
            AstExpression::BooleanLiteral(node) => self.visit_boolean_literal(node),
            AstExpression::NullLiteral(_) => self.visit_null_literal(),
            AstExpression::UndefinedLiteral(_) => self.visit_undefined_literal(),
            AstExpression::ThisExpression(node) => self.visit_this_expression(node),
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
        self.emit(OpCode::JumpIfFalse, &[0]);
        let else_branch_address = self.offset() - 1;
        self.visit_expression(&node.consequent);
        self.emit(OpCode::Jump, &[0]);
        let then_branch_end = self.offset() - 1;
        self.patch_jump_address(else_branch_address, self.offset());
        self.visit_expression(&node.alternative);
        self.patch_jump_address(then_branch_end, self.offset());
    }

    fn visit_array_expression(&mut self, node: &ArrayExpressionNode) {
        node.items.iter().for_each(|x| self.visit_expression(x));
    }

    fn visit_function_expression(&mut self, node: &FunctionExpressionNode) {
        node.arguments.iter().for_each(|x| self.visit_function_argument(x));
        self.visit_statement(&node.body);
    }

    fn visit_undefined_literal(&mut self) {
        let index = self.add_constant(JsValue::Undefined) as u8;
        self.emit(OpCode::PushLiteral, &index.to_ne_bytes());
    }

    fn visit_null_literal(&mut self) {
        let index = self.add_constant(JsValue::Null) as u8;
        self.emit(OpCode::PushLiteral, &index.to_ne_bytes());
    }

    fn visit_this_expression(&mut self, _: &ThisExpressionNode) {}

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
        self.visit_expression(stmt.right.as_ref());
        self.visit_expression(stmt.left.as_ref());
        let opcode = match stmt.operator {
            BinaryOperator::Add => OpCode::Add,
            BinaryOperator::Sub => OpCode::Sub,
            BinaryOperator::Div => OpCode::Div,
            BinaryOperator::Mul => OpCode::Mul,
            BinaryOperator::MulMul => OpCode::MulMul,
            BinaryOperator::LogicalOr => OpCode::Or,
            BinaryOperator::LogicalAnd => OpCode::And,
            BinaryOperator::MoreThan => OpCode::More,
            BinaryOperator::MoreThanOrEqual => OpCode::MoreOrEqual,
            BinaryOperator::LessThan => OpCode::Less,
            BinaryOperator::LessThanOrEqual => OpCode::LessOrEqual,
            BinaryOperator::Equality => OpCode::Eq,
            BinaryOperator::Inequality =>  OpCode::Neq,
        };
        self.emit_opcode(opcode);
    }

    fn visit_boolean_literal(&mut self, node: &BooleanLiteralNode) {
        let opcode = match node.value {
            true => OpCode::PushTrue,
            false => OpCode::PushFalse,
        };
        self.emit_opcode(opcode);
    }

    fn visit_program_statement(&mut self, stmt: &ProgramNode) {
        stmt.statements.iter().for_each(|stmt| self.visit_statement(stmt));
    }

    fn visit_variable_declaration(&mut self, _: &VariableDeclarationNode) {}

    fn visit_identifier_node(&mut self, _: &IdentifierNode) {}
}

pub fn compile_bytecode(ast: &AstStatement) -> CodeBlock {
    // let mut parser = Parser::default();
    // let ast = parser.parse(code)?;
    let mut bytecode_compiler = BytecodeCompiler::new();
    bytecode_compiler.compile(&ast);
    bytecode_compiler.code_block
}