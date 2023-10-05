use crate::interpreter_visitor::{Execute, Interpreter};
use crate::node::GetSpan;
use crate::nodes::AstExpression;
use crate::scanner::{Span, TextSpan, TokenKind};
use crate::value::JsValue;

#[derive(Debug, Clone, PartialEq)]
pub struct AssignmentExpressionNode {
    pub left: Box<AstExpression>,
    pub operator: AssignmentOperator,
    pub right: Box<AstExpression>,
}

impl Execute for AssignmentExpressionNode {
    fn execute(&self, interpreter: &Interpreter) -> Result<JsValue, String> {
        let right_hand_value = self.right.execute(interpreter)?;

        match &self.left.as_ref() {
            AstExpression::Identifier(id_node) => {
                let original_value = interpreter
                    .environment
                    .borrow()
                    .borrow()
                    .get_variable_value(&id_node.id);

                let new_variable_value = match self.operator {
                    AssignmentOperator::AddEqual => &original_value + &right_hand_value,
                    AssignmentOperator::SubEqual => &original_value - &right_hand_value,
                    AssignmentOperator::DivEqual => &original_value / &right_hand_value,
                    AssignmentOperator::MulEqual => &original_value * &right_hand_value,
                    AssignmentOperator::ExponentiationEqual => original_value.exponentiation(&right_hand_value),
                    AssignmentOperator::Equal => Ok(right_hand_value),
                }.unwrap();

                interpreter.environment.borrow()
                    .borrow_mut()
                    .assign_variable(id_node.id.clone(), new_variable_value.clone())?;
                return Ok(new_variable_value);
            }
            AstExpression::MemberExpression(node) => {
                let object = node.object.execute(interpreter)?;
                let key = interpreter.eval_member_expression_key(&node.property, node.computed)?;

                match object {
                    JsValue::Object(object_value) => {
                        let object = object_value;

                        object
                            .borrow_mut()
                            .add_property(key.as_str(), right_hand_value);

                        Ok(JsValue::Object(object))
                    },
                    JsValue::Undefined => Err(format!("Uncaught TypeError: Cannot read properties of undefined (reading '{}')", key).to_string()),
                    _ => Err("Cannot assign: left hand side expression is not an object".to_string())
                }
            }
            _ => todo!(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum AssignmentOperator {
    AddEqual,
    SubEqual,
    DivEqual,
    MulEqual,
    ExponentiationEqual,
    Equal,
}

impl GetSpan for AssignmentExpressionNode {
    fn get_span(&self) -> TextSpan {
        let begin_span = self.left.get_span();
        let end_span = self.right.get_span();

        TextSpan {
            start: Span {
                line: begin_span.start.line,
                row: begin_span.start.row,
            },
            end: Span {
                line: end_span.end.line,
                row: end_span.end.row
            },
        }
    }
}

impl TryFrom<&TokenKind> for AssignmentOperator {
    type Error = String;

    fn try_from(value: &TokenKind) -> Result<Self, Self::Error> {
        match value {
            TokenKind::PlusEqual => Ok(Self::AddEqual),
            TokenKind::MinusEqual => Ok(Self::SubEqual),
            TokenKind::MulEqual => Ok(Self::MulEqual),
            TokenKind::MulMulEqual => Ok(Self::ExponentiationEqual),
            TokenKind::DivEqual => Ok(Self::DivEqual),
            TokenKind::Equal => Ok(Self::Equal),
            _ => Err("Cannot convert token kind to assignment operator".to_string()),
        }
    }
}
