use ariadne::{Color, Label, Report, ReportKind, Source};
use crate::diagnostic::PrintDiagnostic;
use crate::scanner::TextSpan;

#[derive(Debug)]
pub struct ConstantAssigningDiagnostic {
    pub id_span: TextSpan,
}

impl PrintDiagnostic for ConstantAssigningDiagnostic {
    fn print_diagnostic(&self, source: &str) {
        // TODO: add filename
        report_symbol_diagnostic(ReportKind::Error, "assignment to constant variable.", &self.id_span, "a.js", source);
    }
}

#[derive(Debug)]
pub struct UnusedVariableDiagnostic {
    pub variable_name: String,
    pub id_span: TextSpan,
}

impl PrintDiagnostic for UnusedVariableDiagnostic {
    fn print_diagnostic(&self, source: &str) {
        let warning_message = format!("variable '{}' is never used", self.variable_name);
        // TODO: add filename
        report_symbol_diagnostic(ReportKind::Warning, warning_message.as_str(), &self.id_span, "a.js", source);
    }
}

#[derive(Debug)]
pub struct VariableNotDefinedDiagnostic {
    pub variable_name: String,
    pub id_span: TextSpan,
}

impl PrintDiagnostic for VariableNotDefinedDiagnostic {
    fn print_diagnostic(&self, source: &str) {
        let warning_message = format!("variable '{}' is not defined", self.variable_name);
        // TODO: add filename
        report_symbol_diagnostic(ReportKind::Error, warning_message.as_str(), &self.id_span, "a.js", source);
    }
}

#[derive(Debug)]
pub struct MultipleAssignmentDiagnostic {
    pub symbol_name: String,
    pub id_span: TextSpan,
}

impl PrintDiagnostic for MultipleAssignmentDiagnostic {
    fn print_diagnostic(&self, source: &str) {
        let warning_message = format!("identifier '{}' has already been declared", self.symbol_name);
        // TODO: add filename
        report_symbol_diagnostic(ReportKind::Error, warning_message.as_str(), &self.id_span, "a.js", source);
    }
}

fn report_symbol_diagnostic(report_kind: ReportKind, message: &str, span: &TextSpan, filename: &str, source: &str) {
    let color = match report_kind {
        ReportKind::Error => Color::Red,
        _ => Color::Yellow
    };

    Report::build(report_kind, filename, span.start.row)
        .with_message(message)
        .with_label(
            Label::new((filename, span.start.row..span.end.row))
                .with_color(color),
        )
        .finish()
        .print((filename, Source::from(source)))
        .unwrap();
}
