use std::cell::{RefCell};
use std::rc::Rc;
use std::collections::HashMap;
use crate::diagnostic::{Diagnostic, DiagnosticBagRef, DiagnosticKind};
use crate::nodes::*;
use crate::scanner::{TextSpan};
use crate::Source;
use crate::symbol_checker::diagnostics::{ConstantAssigningDiagnostic, ManualImplOfAssignOperationDiagnostic, MultipleAssignmentDiagnostic, UnusedVariableDiagnostic, VariableNotDefinedDiagnostic, WrongBreakContextDiagnostic, WrongThisContextDiagnostic};
use crate::visitor::Visitor;

/// Should traverse ast and find unused variables & assigning to constant variables
pub struct SymbolChecker {
    source: Rc<Source>,
    environment: RefCell<LightEnvironmentRef>,
    diagnostic_bag: DiagnosticBagRef,
    is_inside_this_context: bool,
    break_context_stack: Vec<bool>,
}

impl SymbolChecker {
    pub fn new(source: Rc<Source>, diagnostic_bag: DiagnosticBagRef, globals: Vec<String>) -> Self {
        let global_environment = LightEnvironment::with_symbols(globals);

        Self {
            environment: RefCell::new(LightEnvironment::new(global_environment.into()).into()),
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
                self.report_unused_variable(symbol_name);
            }
        });
    }

    fn report_unused_variable(&self, variable_name: &str) {
        let current_environment = self.environment.borrow();
        let current_environment = current_environment.borrow();

        let symbol = current_environment.symbols.get(variable_name);

        if let Some(symbol) = symbol {
            self.diagnostic_bag.borrow_mut().report_warning(
                Diagnostic::new(DiagnosticKind::UnusedVariable(
                    UnusedVariableDiagnostic { id_span: symbol.span.clone(), variable_name: variable_name.to_string() }
                ), self.source.clone())
            );
        }
    }

    fn report_variable_not_defined(&self, variable_name: &str, span: TextSpan) {
        self.diagnostic_bag.borrow_mut().report_error(
            Diagnostic::new(DiagnosticKind::VariableNotDefined(
                VariableNotDefinedDiagnostic { variable_name: variable_name.to_string(), id_span: span }
            ), self.source.clone())
        );
    }

    fn define_variable(&mut self, symbol_name: &str, is_const: bool, span: TextSpan) {
        let is_already_defined = self.environment.borrow().borrow_mut()
            .define_variable(symbol_name, Symbol { is_const, span: span.clone() });

        if is_already_defined {
            self.diagnostic_bag.borrow_mut().report_error(
                Diagnostic::new(DiagnosticKind::MultipleAssignment(
                    MultipleAssignmentDiagnostic { symbol_name: symbol_name.to_string(), id_span: span }
                ), self.source.clone())
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

impl Into<LightEnvironmentRef> for LightEnvironment {
    fn into(self) -> LightEnvironmentRef {
        Rc::new(RefCell::new(self))
    }
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

    fn with_symbols(symbol_names: Vec<String>) -> Self {
        let symbols = symbol_names.into_iter().map(|x| (x, Symbol { span: TextSpan::default(), is_const: true }));
        Self {
            parent: None,
            symbols: HashMap::from_iter(symbols),
            usages: HashMap::new(),
        }
    }

    /// returns true if variables was already defined
    fn define_variable(&mut self, variable_name: &str, symbol: Symbol) -> bool {
        if self.symbols.contains_key(variable_name) {
            return true;
        }
        self.symbols.insert(variable_name.to_string(), symbol);
        false
    }

    /// returns true if variable is exists
    fn add_usage(&mut self, symbol_name: &str, span: TextSpan) -> bool {
        if self.symbols.contains_key(symbol_name) {
            if self.usages.contains_key(symbol_name) {
                self.usages.get_mut(symbol_name).unwrap().push(span);
            } else {
                self.usages.insert(symbol_name.to_string(), vec![span]);
            }
            return true;
        }

        if let Some(parent) = &self.parent {
            return parent.borrow_mut().add_usage(symbol_name, span);
        }

        return false;
    }

    fn get_symbol_span(&self, variable_name: &str) -> Option<TextSpan> {
        if self.symbols.contains_key(variable_name) {
            return Some(self.symbols.get(variable_name).unwrap().span.clone());
        }

        if let Some(parent) = &self.parent {
            return parent.borrow_mut().get_symbol_span(variable_name);
        }

        None
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

impl Visitor for SymbolChecker {
    // we need to visit initializer first cause it can access a variable with the same name
    fn visit_variable_declaration(&mut self, stmt: &VariableDeclarationNode) {
        if let Some(value) = &stmt.value {
            self.visit_expression(value);
        }

        let variable_name = &stmt.id.id;
        self.define_variable(&variable_name, matches!(stmt.kind, VariableDeclarationKind::Const), stmt.id.get_span());
    }

    fn visit_block_statement(&mut self, stmt: &BlockStatementNode) {
        self.set_environment(self.create_new_environment());
        stmt.statements.iter().for_each(|x| self.visit_statement(x));
        self.pop_environment();
    }

    fn visit_template_string_literal_expression(&mut self, node: &TemplateStringLiteralNode) {
        node.elements.iter().for_each(|x| {
            if let TemplateElement::Expression(expression) = x {
                self.visit_expression(expression)
            }
            // match x {
            //     TemplateElement::Raw(_) => {}
            //     TemplateElement::Expression(expression) => self.visit_expression(expression)
            // }
        });
    }

    fn visit_assignment_expression(&mut self, stmt: &AssignmentExpressionNode) {
        // check for manual implementation of assign operation
        // TODO: extract a function
        if let AstExpression::BinaryExpression(expr) = stmt.right.as_ref() {
            if expr.left == stmt.left {
                self.diagnostic_bag.borrow_mut().report_warning(
                    Diagnostic::new(DiagnosticKind::ManualImplOfAssignOperation(
                        ManualImplOfAssignOperationDiagnostic { span: stmt.get_span() }
                    ), self.source.clone())
                );
            }
        }

        match &stmt.left.as_ref() {
            AstExpression::Identifier(id_node) => {
                self.visit_identifier_node(id_node);

                let variable_name = &id_node.id;

                let diagnostic = self.environment.borrow()
                    .borrow_mut()
                    .assign_variable(variable_name);

                if diagnostic.is_some() {
                    match diagnostic.unwrap() {
                        AssignVariableResult::ConstantAssigning => {
                            let declaration_span = self.environment.borrow()
                                .borrow_mut().get_symbol_span(variable_name).unwrap();

                            self.diagnostic_bag.borrow_mut().report_error(
                                Diagnostic::new(DiagnosticKind::ConstantAssigning(
                                    ConstantAssigningDiagnostic {
                                        variable_name: variable_name.clone(),
                                        declaration_span,
                                        id_span: stmt.left.get_span(),
                                    }
                                ), self.source.clone())
                            );
                        }
                        AssignVariableResult::VariableNotDefined => {
                            self.report_variable_not_defined(&id_node.id, stmt.left.get_span());
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

    fn visit_member_expression(&mut self, stmt: &MemberExpressionNode) {
        self.visit_expression(&stmt.object);
    }

    fn visit_identifier_node(&mut self, stmt: &IdentifierNode) {
        let symbol_name = stmt.id.as_str();
        let is_symbol_exists = self.environment.borrow().borrow_mut().add_usage(stmt.id.as_str(), stmt.get_span());

        if !is_symbol_exists {
            self.report_variable_not_defined(symbol_name, stmt.get_span());
        }
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

        self.define_variable(stmt.function_signature.name.id.as_str(), false, stmt.function_signature.name.get_span());
        for arg in &stmt.function_signature.arguments {
            self.define_variable(&arg.name.id, false, arg.name.get_span());
        }
        self.visit_function_signature(&stmt.function_signature);

        self.is_inside_this_context = false;
        self.pop_break_context();
    }

    fn visit_this_expression(&mut self, node: &ThisExpressionNode) {
        if !self.is_inside_this_context {
            self.diagnostic_bag.borrow_mut().report_error(
                Diagnostic::new(DiagnosticKind::WrongThisContext(
                    WrongThisContextDiagnostic { span: node.token.span.clone() }
                ), self.source.clone())
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

    fn visit_break_statement(&mut self, node: &BreakStatementNode) {
        let break_context_state = self.break_context_stack.last();
        let is_inside_break_context = break_context_state.is_some() && *break_context_state.unwrap();

        if !is_inside_break_context {
            self.diagnostic_bag.borrow_mut().report_error(
                Diagnostic::new(DiagnosticKind::WrongBreakContext(
                    WrongBreakContextDiagnostic { span: node.0.span.clone() }
                ), self.source.clone())
            );
        }
    }
}

mod tests {
    use std::cell::RefCell;
    use std::rc::Rc;
    use crate::diagnostic::{Diagnostic, DiagnosticBag, DiagnosticBagRef, DiagnosticKind};
    use crate::globals::get_globals;
    use crate::parser::Parser;
    use crate::source::Source;
    use crate::symbol_checker::symbol_checker::SymbolChecker;

    fn parse_and_collect_diagnostics(code: &str) -> DiagnosticBagRef {
        let source = Rc::new(Source::inline_source(code.to_string()));
        let diagnostic_bag_ref = Rc::new(RefCell::new(DiagnosticBag::new()));
        let globals = get_globals();
        let global_names: Vec<String> = globals.iter().map(|x| x.name.clone()).collect();
        let mut symbol_checker = SymbolChecker::new(source.clone(), Rc::clone(&diagnostic_bag_ref), global_names);
        let ast = Parser::parse_code_to(source).expect("Parsing error");
        symbol_checker.check_symbols(&ast);
        symbol_checker.diagnostic_bag
    }

    #[test]
    fn report_unused_variables() {
        let code = "let a = 5;";
        let diagnostic_bag = parse_and_collect_diagnostics(&code);
        assert_eq!(diagnostic_bag.borrow().warnings.len(), 1);
        assert_eq!(diagnostic_bag.borrow().errors.len(), 0);
        assert!(matches!(
            diagnostic_bag.borrow().warnings[0],
            Diagnostic { kind: DiagnosticKind::UnusedVariable(ref n), .. } if n.variable_name == "a"
        ));
    }

    #[test]
    fn not_report_unused_variables() {
        let code = "let a = 5; { a; }";
        let diagnostic_bag = parse_and_collect_diagnostics(&code);
        assert_eq!(diagnostic_bag.borrow().warnings.len(), 0);
        assert_eq!(diagnostic_bag.borrow().errors.len(), 0);
    }

    #[test]
    fn report_unused_function() {
        let code = "function add() {}";
        let diagnostic_bag = parse_and_collect_diagnostics(&code);
        assert_eq!(diagnostic_bag.borrow().warnings.len(), 1);
        assert_eq!(diagnostic_bag.borrow().errors.len(), 0);
        assert!(matches!(
            diagnostic_bag.borrow().warnings[0],
            Diagnostic { kind: DiagnosticKind::UnusedVariable(ref n), .. } if n.variable_name == "add"
        ));
    }

    #[test]
    fn report_reassigning_constant() {
        let code = "const a = 5; a = 10;";
        let diagnostic_bag = parse_and_collect_diagnostics(&code);
        assert_eq!(diagnostic_bag.borrow().warnings.len(), 0);
        assert_eq!(diagnostic_bag.borrow().errors.len(), 1);
        assert!(matches!(
            diagnostic_bag.borrow().errors[0],
            Diagnostic { kind: DiagnosticKind::ConstantAssigning(ref n), .. } if n.variable_name == "a"
        ));
    }

    #[test]
    fn not_report_globals() {
        let code = "print(Math.cos(Math.sin(Math.PI)));";
        let diagnostic_bag = parse_and_collect_diagnostics(&code);
        assert_eq!(diagnostic_bag.borrow().warnings.len(), 0);
        assert_eq!(diagnostic_bag.borrow().errors.len(), 0);
    }
}
