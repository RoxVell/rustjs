use std::rc::Rc;
use std::cell::RefCell;
use crate::symbol_checker::diagnostics::{ConstantAssigningDiagnostic, ManualImplOfAssignOperationDiagnostic, MultipleAssignmentDiagnostic, UnusedVariableDiagnostic, VariableNotDefinedDiagnostic, WrongBreakContextDiagnostic, WrongThisContextDiagnostic};

pub struct DiagnosticBag<'a> {
    pub warnings: Vec<Diagnostic<'a>>,
    pub errors: Vec<Diagnostic<'a>>,
}

pub type DiagnosticBagRef<'a> = Rc<RefCell<DiagnosticBag<'a>>>;

impl<'a> DiagnosticBag<'a> {
    pub fn new() -> Self {
        Self {
            warnings: vec![],
            errors: vec![],
        }
    }

    pub fn report_error(&mut self, diagnostic: Diagnostic<'a>) {
        self.errors.push(diagnostic);
    }

    pub fn report_warning(&mut self, diagnostic: Diagnostic<'a>) {
        self.warnings.push(diagnostic);
    }
}

#[derive(Debug)]
pub enum DiagnosticKind {
    UnusedVariable(UnusedVariableDiagnostic),
    ConstantAssigning(ConstantAssigningDiagnostic),
    VariableNotDefined(VariableNotDefinedDiagnostic),
    MultipleAssignment(MultipleAssignmentDiagnostic),
    WrongThisContext(WrongThisContextDiagnostic),
    WrongBreakContext(WrongBreakContextDiagnostic),
    ManualImplOfAssignOperation(ManualImplOfAssignOperationDiagnostic),
}

#[derive(Debug)]
pub struct Diagnostic<'a> {
    kind: DiagnosticKind,
    source: &'a str
}

impl<'a> Diagnostic<'a> {
    pub(crate) fn new(kind: DiagnosticKind, source: &'a str) -> Self {
        Self {
            kind,
            source,
        }
    }

    pub fn print_diagnostic(&self) {
        match &self.kind {
            DiagnosticKind::UnusedVariable(diagnostic) => diagnostic.print_diagnostic(self.source),
            DiagnosticKind::ConstantAssigning(diagnostic) => diagnostic.print_diagnostic(self.source),
            DiagnosticKind::VariableNotDefined(diagnostic) => diagnostic.print_diagnostic(self.source),
            DiagnosticKind::MultipleAssignment(diagnostic) => diagnostic.print_diagnostic(self.source),
            DiagnosticKind::WrongThisContext(diagnostic) => diagnostic.print_diagnostic(self.source),
            DiagnosticKind::WrongBreakContext(diagnostic) => diagnostic.print_diagnostic(self.source),
            DiagnosticKind::ManualImplOfAssignOperation(diagnostic) => diagnostic.print_diagnostic(self.source),
        }
    }
}

pub trait PrintDiagnostic {
    fn print_diagnostic(&self, source: &str);
}
