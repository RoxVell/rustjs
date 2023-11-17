use std::rc::Rc;
use std::cell::RefCell;
use enum_dispatch::enum_dispatch;
use crate::Source;
use crate::symbol_checker::diagnostics::{ConstantAssigningDiagnostic, ManualImplOfAssignOperationDiagnostic, MultipleAssignmentDiagnostic, UnusedVariableDiagnostic, VariableNotDefinedDiagnostic, WrongBreakContextDiagnostic, WrongThisContextDiagnostic};

pub struct DiagnosticBag {
    pub warnings: Vec<Diagnostic>,
    pub errors: Vec<Diagnostic>,
}

pub type DiagnosticBagRef = Rc<RefCell<DiagnosticBag>>;

impl DiagnosticBag {
    pub fn new() -> Self {
        Self {
            warnings: vec![],
            errors: vec![],
        }
    }

    pub fn report_error(&mut self, diagnostic: Diagnostic) {
        self.errors.push(diagnostic);
    }

    pub fn report_warning(&mut self, diagnostic: Diagnostic) {
        self.warnings.push(diagnostic);
    }
}

#[derive(Debug)]
#[enum_dispatch(PrintDiagnostic)]
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
pub struct Diagnostic {
    pub(super) kind: DiagnosticKind,
    source: Rc<Source>,
}

impl Diagnostic {
    pub(crate) fn new(kind: DiagnosticKind, source: Rc<Source>) -> Self {
        Self {
            kind,
            source,
        }
    }

    pub fn print_diagnostic(&self) {
        self.kind.print_diagnostic(self.source.as_ref())
    }
}

#[enum_dispatch]
pub trait PrintDiagnostic {
    fn print_diagnostic(&self, source: &Source);
}
