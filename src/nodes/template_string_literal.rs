use std::fmt::{Debug, Formatter};
use crate::interpreter::ast_interpreter::{Interpreter};
use crate::nodes::{AstExpression, Execute, StringLiteralNode, TextSpan};
use crate::scanner::{Span, Token, TokenKind};
use crate::value::JsValue;

#[derive(Debug, Clone, PartialEq)]
pub enum TemplateElement {
    Raw(String),
    Expression(AstExpression),
}

#[derive(Clone, PartialEq)]
pub struct TemplateStringLiteralNode {
    pub elements: Vec<TemplateElement>,
    pub token: Token,
}

impl TemplateStringLiteralNode {

}

// fn transform_template_string_to_binary_expression(template_node: &TemplateStringLiteralNode) -> AstExpression {
//     match template_node.elements.len() {
//         0 => {
//             AstExpression::StringLiteral(
//                 StringLiteralNode {
//                     token: Token {
//                         token: TokenKind::String("".to_string()),
//                         span: TextSpan {
//                             start: Span { line: 0, row: 0 }, end: Span { line: 0, row: 0 }
//                         }
//                     },
//                     value: "".to_string()
//                 }
//             )
//         },
//         1 => {
//
//         },
//         _ => {
//
//         }
//     }
//
//     todo!()
//     // let mut expression
// }

impl Execute for TemplateStringLiteralNode {
    fn execute(&self, interpreter: &Interpreter) -> Result<JsValue, String> {
        let mut result = String::new();

        for element in &self.elements {
            match element {
                TemplateElement::Raw(raw_string) => result.push_str(raw_string),
                TemplateElement::Expression(expression) => {
                    result.push_str(&expression.execute(interpreter).unwrap().to_js_like_string());
                }
            }
        }

        Ok(JsValue::String(result))
    }
}

impl Debug for TemplateStringLiteralNode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "<template_string>")
    }
}
