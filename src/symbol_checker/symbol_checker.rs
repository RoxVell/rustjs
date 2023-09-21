use std::cell::RefCell;
use std::rc::Rc;
use std::collections::HashMap;
use crate::diagnostic::{Diagnostic, DiagnosticBagRef, DiagnosticKind};
use crate::node::{AssignmentExpressionNode, AstExpression, AstStatement, BlockStatementNode, ClassDeclarationNode, ForStatementNode, FunctionDeclarationNode, GetSpan, IdentifierNode, VariableDeclarationKind, VariableDeclarationNode, WhileStatementNode};
use crate::scanner::{TextSpan, Token};
use crate::symbol_checker::diagnostics::{ConstantAssigningDiagnostic, MultipleAssignmentDiagnostic, UnusedVariableDiagnostic, VariableNotDefinedDiagnostic, WrongBreakContextDiagnostic, WrongThisContextDiagnostic};
use crate::visitor::Visitor;

/// Should traverse ast and find unused variables & assigning to constant variables
pub struct SymbolChecker<'a> {
    source: &'a str,
    environment: RefCell<LightEnvironmentRef>,
    diagnostic_bag: DiagnosticBagRef<'a>,
    is_inside_this_context: bool,
    break_context_stack: Vec<bool>,
}

impl<'a> SymbolChecker<'a> {
    pub fn new(source: &'a str, diagnostic_bag: DiagnosticBagRef<'a>) -> Self {
        Self {
            environment: RefCell::new(Rc::new(RefCell::new(LightEnvironment::default()))),
            source,
            diagnostic_bag,
            is_inside_this_context: false,
            break_context_stack: vec![],
        }
    }

    pub fn check_symbols(&mut self, stmt: &AstStatement) {
        self.visit_statement(stmt);
        self.check_unused_symbols();
    }

    fn check_unused_symbols(&self) {
        let current_environment = self.environment.borrow();
        let current_environment = current_environment.borrow();

        current_environment.symbols.keys().for_each(|symbol_name| {
            let usage = current_environment.usages.get(symbol_name);

            if usage.is_none() {
                let symbol = current_environment.symbols.get(symbol_name);

                if let Some(symbol) = symbol {
                    self.diagnostic_bag.borrow_mut().report_warning(
                        Diagnostic::new(DiagnosticKind::UnusedVariable(
                            UnusedVariableDiagnostic { id_span: symbol.span.clone(), variable_name: symbol_name.clone() }
                        ), self.source)
                    );
                }
            }
        });
    }

    fn define_variable(&mut self, symbol_name: &str, is_const: bool, span: TextSpan) {
        let error = self.environment.borrow().borrow_mut()
            .define_variable(symbol_name, Symbol { is_const, span: span.clone() });

        if error.is_some() {
            self.diagnostic_bag.borrow_mut().report_error(
                Diagnostic::new(DiagnosticKind::MultipleAssignment(
                    MultipleAssignmentDiagnostic { symbol_name: symbol_name.to_string(), id_span: span }
                ), self.source)
            );
        }
    }

    fn create_new_environment(&self) -> LightEnvironment {
        return LightEnvironment::new(Rc::clone(&self.environment.borrow().clone()));
    }

    fn set_environment(&self, environment: LightEnvironment) {
        self.environment.replace(Rc::new(RefCell::new(environment)));
    }

    fn pop_environment(&mut self) {
        self.check_unused_symbols();

        let parent_environment = self
            .environment
            .borrow()
            .borrow()
            .get_parent()
            .unwrap()
            .borrow()
            .to_owned();

        self.set_environment(parent_environment);
    }

    fn enter_break_context(&mut self) {
        self.break_context_stack.push(true);
    }

    fn out_break_context(&mut self) {
        self.break_context_stack.push(false);
    }

    fn pop_break_context(&mut self) {
        self.break_context_stack.pop();
    }
}

#[derive(Debug, Clone)]
struct Symbol {
    span: TextSpan,
    is_const: bool
}

#[derive(Default, Debug, Clone)]
struct LightEnvironment {
    parent: Option<LightEnvironmentRef>,
    symbols: HashMap<String, Symbol>,
    usages: HashMap<String, Vec<TextSpan>>,
}

type LightEnvironmentRef = Rc<RefCell<LightEnvironment>>;

#[derive(Debug)]
enum AssignVariableResult {
    ConstantAssigning,
    VariableNotDefined,
}

impl LightEnvironment {
    fn new(parent: LightEnvironmentRef) -> Self {
        Self {
            parent: Some(parent),
            symbols: HashMap::new(),
            usages: HashMap::new(),
        }
    }

    fn define_variable(&mut self, variable_name: &str, symbol: Symbol) -> Option<()> {
        if self.symbols.contains_key(variable_name) {
            return Some(());
        }
        self.symbols.insert(variable_name.to_string(), symbol);
        return None;
    }

    fn add_usage(&mut self, variable_name: &str, span: TextSpan) {
        if self.symbols.contains_key(variable_name) {
            if self.usages.contains_key(variable_name) {
                self.usages.get_mut(variable_name).unwrap().push(span);
            } else {
                self.usages.insert(variable_name.to_string(), vec![span]);
            }
            return ();
        }

        if let Some(parent) = &self.parent {
            parent.borrow_mut().add_usage(variable_name, span);
        }
    }

    fn assign_variable(&mut self, variable_name: &str) -> Option<AssignVariableResult> {
        if self.symbols.contains_key(variable_name) {
            let symbol = self.symbols.get(variable_name).unwrap();

            return match symbol.is_const {
                true => Some(AssignVariableResult::ConstantAssigning),
                false => None,
            };
        }

        if let Some(parent) = &self.parent {
            return parent.borrow_mut().assign_variable(variable_name);
        }

        if !self.symbols.contains_key(variable_name) {
            return Some(AssignVariableResult::VariableNotDefined);
        }

        return None;
    }

    pub fn get_parent(&self) -> Option<LightEnvironmentRef> {
        self.parent.as_ref().map(|x| Rc::clone(x))
    }
}

impl<'a> Visitor for SymbolChecker<'a> {
    fn visit_variable_declaration(&mut self, stmt: &VariableDeclarationNode) {
        let variable_name = &stmt.id.id;
        self.define_variable(&variable_name, matches!(stmt.kind, VariableDeclarationKind::Const), stmt.id.get_span());

        if let Some(value) = &stmt.value {
            self.visit_expression(value);
        }
    }

    fn visit_block_statement(&mut self, stmt: &BlockStatementNode) {
        self.set_environment(self.create_new_environment());
        stmt.statements.iter().for_each(|x| self.visit_statement(x));
        self.pop_environment();
    }

    fn visit_assignment_expression(&mut self, stmt: &AssignmentExpressionNode) {
        match &stmt.left.as_ref() {
            AstExpression::Identifier(id_node) => {
                self.visit_identifier_node(id_node);

                let diagnostic = self.environment.borrow()
                    .borrow_mut()
                    .assign_variable(&id_node.id);

                if diagnostic.is_some() {
                    match diagnostic.unwrap() {
                        AssignVariableResult::ConstantAssigning => {
                            self.diagnostic_bag.borrow_mut().report_error(
                                Diagnostic::new(DiagnosticKind::ConstantAssigning(
                                    ConstantAssigningDiagnostic { id_span: stmt.left.get_span() }
                                ), self.source)
                            );
                        }
                        AssignVariableResult::VariableNotDefined => {
                            self.diagnostic_bag.borrow_mut().report_error(
                                Diagnostic::new(DiagnosticKind::VariableNotDefined(
                                    VariableNotDefinedDiagnostic { variable_name: id_node.id.clone(), id_span: stmt.left.get_span() }
                                ), self.source)
                            );
                        }
                    }
                }
            }
            AstExpression::MemberExpression(node) => {
                self.visit_member_expression(node);
            }
            _ => todo!(),
        }
    }

    fn visit_identifier_node(&mut self, stmt: &IdentifierNode) {
        self.environment.borrow().borrow_mut().add_usage(stmt.id.as_str(), stmt.get_span())
    }

    fn visit_class_declaration(&mut self, stmt: &ClassDeclarationNode) {
        self.define_variable(&stmt.name.id, false, stmt.name.get_span());

        if let Some(parent) = &stmt.parent {
            self.visit_identifier_node(parent);
        }

        self.is_inside_this_context = true;
        stmt.methods.iter().for_each(|x| self.visit_class_method(x));
        self.is_inside_this_context = false;
    }

    fn visit_function_declaration(&mut self, stmt: &FunctionDeclarationNode) {
        self.out_break_context();
        self.is_inside_this_context = true;
        self.visit_function_signature(&stmt.function_signature);
        self.is_inside_this_context = false;
        self.define_variable(stmt.function_signature.name.id.as_str(), false, stmt.function_signature.name.get_span());
        self.pop_break_context();
    }

    fn visit_this_expression(&mut self, token: &Token) {
        if !self.is_inside_this_context {
            self.diagnostic_bag.borrow_mut().report_error(
                Diagnostic::new(DiagnosticKind::WrongThisContext(
                    WrongThisContextDiagnostic { span: token.span.clone() }
                ), self.source)
            );
        }
    }

    fn visit_while_statement(&mut self, node: &WhileStatementNode) {
        self.enter_break_context();
        self.visit_expression(&node.condition);
        self.visit_statement(&node.body);
        self.pop_break_context();
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

        self.enter_break_context();
        self.visit_statement(&stmt.body);
        self.pop_break_context();
    }

    fn visit_break_statement(&mut self, token: &Token) {
        let break_context_state = self.break_context_stack.last();
        let is_inside_break_context = break_context_state.is_some() && *break_context_state.unwrap();

        if !is_inside_break_context {
            self.diagnostic_bag.borrow_mut().report_error(
                Diagnostic::new(DiagnosticKind::WrongBreakContext(
                    WrongBreakContextDiagnostic { span: token.span.clone() }
                ), self.source)
            );
        }
    }
}
