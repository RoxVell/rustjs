use std::mem;
use crate::bytecode::opcodes::Opcode;
use crate::nodes::*;
use crate::value::function::JsFunction;
use crate::value::JsValue;
use crate::value::object::JsObject;
use crate::visitor::Visitor;

#[derive(Debug)]
pub struct GlobalVariable {
    pub name: String,
    pub value: JsValue,
}

impl GlobalVariable {
    pub fn new(name: String, value: JsValue) -> Self {
        Self {
            name,
            value
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct LocalVariable {
    pub name: String,
    scope_level: u8,
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct CodeBlock {
    pub name: String,
    pub bytecode: Vec<u8>,
    pub constants: Vec<JsValue>,
    pub locals: Vec<LocalVariable>,
    pub scope_level: u8,
    pub arity: u8,
}

impl CodeBlock {
    pub fn new(name: String, arity: u8) -> Self {
        Self {
            name,
            bytecode: vec![],
            constants: vec![],
            locals: vec![],
            scope_level: 0,
            arity,
        }
    }
}

pub struct BytecodeCompiler {
    pub code_block: CodeBlock,
    pub code_blocks: Vec<CodeBlock>,
    pub globals: Vec<GlobalVariable>,
}

impl BytecodeCompiler {
    pub fn new(globals: Vec<GlobalVariable>) -> Self {
        Self {
            code_block: CodeBlock::new("main".to_string(), 0),
            code_blocks: vec![],
            globals
        }
    }

    pub fn compile(&mut self, ast: &AstStatement) {
        self.code_block = CodeBlock::new("main".to_string(), 0);
        self.visit_statement(ast);
        let code_block = mem::replace(&mut self.code_block, CodeBlock::default());
        self.code_blocks.push(code_block);
    }

    pub fn get_globals(&self) -> &[GlobalVariable] {
        &self.globals
    }

    fn add_global(&mut self, name: String, value: JsValue) -> u8 {
        self.globals.push(GlobalVariable { name, value });
        (self.globals.len() - 1) as u8
    }

    fn get_global_variable_index(&mut self, variable_name: &str) -> u8 {
        self.globals.iter()
            .rposition(|x| x.name == variable_name)
            .expect(format!("Failed to get global variable index for variable name: {}", variable_name).as_str()) as u8
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

    fn push_literal(&mut self, value: JsValue) {
        let index = self.add_constant(value) as u8;
        self.emit(Opcode::PushLiteral, &index.to_ne_bytes());
    }

    fn add_local_variable(&mut self, variable_name: String) -> u8 {
        self.code_block.locals.push(LocalVariable { name: variable_name, scope_level: self.code_block.scope_level });
        (self.code_block.locals.len() - 1) as u8
    }

    fn get_local_variable_index(&mut self, variable_name: &str) -> Option<u8> {
        self.code_block.locals.iter()
            .rposition(|x| x.name == variable_name)
            .map(|x| x as u8)
    }

    fn emit_opcode(&mut self, opcode: Opcode) {
        self.code_block.bytecode.push(opcode as u8);
    }

    fn emit(&mut self, opcode: Opcode, operands: &[u8]) {
        self.emit_opcode(opcode);
        self.emit_operands(operands);
    }

    fn emit_operands(&mut self, operands: &[u8]) {
        for operand in operands {
            self.code_block.bytecode.push(*operand);
        }
    }

    fn enter_scope(&mut self) {
        self.code_block.scope_level += 1;
    }

    fn exit_scope(&mut self) {
        let n_pop = self.get_n_pop();

        if n_pop > 0 {
            self.emit(Opcode::ExitScope, &[n_pop]);
        }

        self.code_block.scope_level -= 1;
    }

    fn get_n_pop(&mut self) -> u8 {
        let mut n_pop = 0;

        self.code_block.locals.retain(|x| {
            let should_be_deleted = x.scope_level == self.code_block.scope_level;
            if should_be_deleted {
                n_pop += 1;
            }
            // TODO: uncomment this when i understand how to print variable name with bytecode_printer
            // !should_be_deleted
            true
        });

        if self.code_block.name != "main" {
            n_pop += self.code_block.arity + 1;
        }

        n_pop
    }

    fn offset(&self) -> usize {
        self.code_block.bytecode.len()
    }

    fn patch_jump_address(&mut self, src: usize, dest: usize) {
        self.code_block.bytecode[src] = dest as u8;
    }

    fn visit_block(&mut self, statements: &[AstStatement]) {
        self.enter_scope();
        statements.iter().for_each(|stmt| {
            self.visit_statement(stmt);
            // only an expression statement produce a value
            let should_pop = matches!(stmt, AstStatement::ExpressionStatement(_));
            if should_pop {
                self.emit_opcode(Opcode::Pop);
            }
        });
        self.exit_scope();
    }

    fn push_member_expression_key(
        &mut self,
        node: &AstExpression,
        computed: bool,
    ) {
        if computed {
            // todo!("implement computed properties")
            self.visit_expression(node);
            // // let computed_key = self.po;
            //
            // return match computed_key {
            //     JsValue::String(value) => Ok(value),
            //     JsValue::Number(value) => Ok(value.to_string()),
            //     _ => Err("".to_string()),
            // };
        } else {
            let key = match node {
                AstExpression::StringLiteral(value) => Ok(value.value.clone()),
                AstExpression::NumberLiteral(node) => Ok(node.value.to_string()),
                AstExpression::Identifier(node) => Ok(node.id.clone()),
                _ => Err("Object key should be an identifier".to_string()),
            }.unwrap();

            self.push_literal(JsValue::from(key));
        }
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
        let start_while_loop_offset = self.offset();
        self.visit_expression(&node.condition);
        self.emit(Opcode::JumpIfFalse, &[0]);
        let jump_off_loop_address = self.offset() - 1;
        self.visit_statement(&node.body);
        self.emit(Opcode::Jump, &[start_while_loop_offset as u8]);
        self.patch_jump_address(jump_off_loop_address, self.offset());
    }

    fn visit_return_statement(&mut self, node: &ReturnStatementNode) {
        self.visit_expression(&node.expression);
        self.emit_opcode(Opcode::Return);
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
        let prev_code_block = mem::replace(
            &mut self.code_block,
            CodeBlock::new(
                stmt.function_signature.name.id.clone(),
                stmt.function_signature.arguments.len() as u8
            )
        );

        self.add_local_variable(stmt.function_signature.name.id.clone());

        for arg in stmt.function_signature.arguments.iter() {
            self.add_local_variable(arg.name.id.clone());
        }

        self.visit_statement(stmt.function_signature.body.as_ref());
        let fn_value: JsValue = JsFunction::Bytecode(self.code_block.clone()).into();
        let co = mem::replace(&mut self.code_block, prev_code_block);
        let constant_idx = self.add_constant(fn_value);
        self.emit(Opcode::PushLiteral, &[constant_idx as u8]);
        let idx = self.add_local_variable(stmt.function_signature.name.id.clone());
        self.emit(Opcode::SetVar, &[idx]);
        self.code_blocks.push(co);
    }

    fn visit_function_signature(&mut self, stmt: &FunctionSignature) {
        self.visit_identifier_node(&stmt.name);
        stmt.arguments.iter().for_each(|x| self.visit_function_argument(x));
        self.visit_statement(&stmt.body);
    }

    fn visit_function_argument(&mut self, stmt: &FunctionArgument) {
        // self.visit_identifier_node(&stmt.name);
        // if let Some(value) = &stmt.default_value {
        //     self.visit_expression(value);
        // }
    }

    fn visit_block_statement(&mut self, stmt: &BlockStatementNode) {
        self.visit_block(&stmt.statements);
    }

    fn visit_if_statement(&mut self, stmt: &IfStatementNode) {
        self.visit_expression(&stmt.condition);

        self.emit(Opcode::JumpIfFalse, &[0]);
        let else_branch_address = self.offset() - 1;

        self.visit_statement(&stmt.then_branch);
        if stmt.else_branch.is_some() {
            self.emit(Opcode::Jump, &[0]);
        }
        let then_branch_end = self.offset() - 1;
        self.patch_jump_address(else_branch_address, self.offset());

        if let Some(else_branch) = &stmt.else_branch {
            self.visit_statement(else_branch);
            self.patch_jump_address(then_branch_end, self.offset());
        }
    }

    fn visit_expression_statement(&mut self, stmt: &AstExpression) {
        self.visit_expression(stmt);
    }

    fn visit_string_literal(&mut self, node: &StringLiteralNode) {
        self.push_literal(JsValue::String(node.value.clone()));
    }

    fn visit_number_literal(&mut self, node: &NumberLiteralNode) {
        self.push_literal(JsValue::Number(node.value));
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
        self.emit(Opcode::JumpIfFalse, &[0]);
        let else_branch_address = self.offset() - 1;
        self.visit_expression(&node.consequent);
        self.emit(Opcode::Jump, &[0]);
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
        self.push_literal(JsValue::Undefined);
    }

    fn visit_null_literal(&mut self) {
        self.push_literal(JsValue::Null);
    }

    fn visit_this_expression(&mut self, _: &ThisExpressionNode) {}

    fn visit_object_expression(&mut self, node: &ObjectExpressionNode) {
        self.push_literal(JsObject::empty().to_js_value());

        for property in &node.properties {
            self.push_member_expression_key(&property.key, property.computed);
            // self.push_literal(JsValue::from(key));
            self.visit_expression(property.value.as_ref());
            self.emit_opcode(Opcode::SetProp);
        }
    }

    fn visit_object_property(&mut self, node: &ObjectPropertyNode) {
        self.visit_expression(&node.value);
        self.visit_expression(&node.key);
    }

    fn visit_member_expression(&mut self, stmt: &MemberExpressionNode) {
        self.visit_expression(&stmt.object);
        self.push_member_expression_key(&stmt.property, stmt.computed);
        self.emit_opcode(Opcode::GetProp);
    }

    fn visit_new_expression(&mut self, stmt: &NewExpressionNode) {
        self.visit_expression(&stmt.callee);
        stmt.arguments.iter().for_each(|x| self.visit_expression(x));
    }

    fn visit_call_expression(&mut self, stmt: &CallExpressionNode) {
        self.visit_expression(&stmt.callee);
        stmt.params.iter().for_each(|x| self.visit_expression(x));
        self.emit(Opcode::Call, &[stmt.params.len() as u8]);
    }

    fn visit_assignment_expression(&mut self, stmt: &AssignmentExpressionNode) {
        if matches!(stmt.operator, AssignmentOperator::Equal) {
            self.visit_expression(&stmt.right);
        } else {
            self.visit_expression(&stmt.right);
            self.visit_expression(&stmt.left);
            let opcode = match stmt.operator {
                AssignmentOperator::AddEqual => Opcode::Add,
                AssignmentOperator::SubEqual => Opcode::Sub,
                AssignmentOperator::DivEqual => Opcode::Div,
                AssignmentOperator::MulEqual => Opcode::Mul,
                AssignmentOperator::ExponentiationEqual => Opcode::MulMul,
                _ => unreachable!()
            };
            self.emit_opcode(opcode);
        }

        // TODO: support only simple assignments with identifier on the lhs
        if let AstExpression::Identifier(ident) = stmt.left.as_ref() {
            let idx = self.get_local_variable_index(&ident.id).unwrap();
            self.emit(Opcode::SetVar, &[idx]);
        } else {
            unimplemented!()
        }
    }

    fn visit_binary_expression(&mut self, stmt: &BinaryExpressionNode) {
        self.visit_expression(stmt.right.as_ref());
        self.visit_expression(stmt.left.as_ref());
        let opcode = match stmt.operator {
            BinaryOperator::Add => Opcode::Add,
            BinaryOperator::Sub => Opcode::Sub,
            BinaryOperator::Div => Opcode::Div,
            BinaryOperator::Mul => Opcode::Mul,
            BinaryOperator::MulMul => Opcode::MulMul,
            BinaryOperator::LogicalOr => Opcode::Or,
            BinaryOperator::LogicalAnd => Opcode::And,
            BinaryOperator::MoreThan => Opcode::More,
            BinaryOperator::MoreThanOrEqual => Opcode::MoreOrEqual,
            BinaryOperator::LessThan => Opcode::Less,
            BinaryOperator::LessThanOrEqual => Opcode::LessOrEqual,
            BinaryOperator::Equality => Opcode::Eq,
            BinaryOperator::Inequality =>  Opcode::Neq,
        };
        self.emit_opcode(opcode);
    }

    fn visit_boolean_literal(&mut self, node: &BooleanLiteralNode) {
        let opcode = match node.value {
            true => Opcode::PushTrue,
            false => Opcode::PushFalse,
        };
        self.emit_opcode(opcode);
    }

    fn visit_program_statement(&mut self, stmt: &ProgramNode) {
        self.visit_block(&stmt.statements);
        // stmt.statements.iter().for_each(|stmt| self.visit_statement(stmt));
    }

    fn visit_variable_declaration(&mut self, node: &VariableDeclarationNode) {
        // TODO: maybe we need to push undefined on top of the stack if there is no initializer?
        if let Some(initializer) = &node.value {
            self.visit_expression(initializer);
        }

        self.add_local_variable(node.id.id.clone());
        // println!("visit_variable_declaration: {idx}");
        // self.emit(Opcode::InitVar, &[idx]);
    }

    fn visit_identifier_node(&mut self, node: &IdentifierNode) {
        let local_idx = self.get_local_variable_index(&node.id);

        if let Some(idx) = local_idx {
            self.emit(Opcode::GetVar, &[idx]);
            return;
        }

        let global_idx = self.get_global_variable_index(&node.id);
        self.emit(Opcode::GetGlobal, &[global_idx]);
    }
}

// pub fn compile_bytecode(ast: &AstStatement) -> CodeBlock {
//     // let mut parser = Parser::default();
//     // let ast = parser.parse(code)?;
//     let mut bytecode_compiler = BytecodeCompiler::new();
//     bytecode_compiler.compile(&ast);
//     bytecode_compiler.code_block
// }