use std::rc::Rc;
use crate::interpreter_visitor::{Execute, Interpreter};
use crate::nodes::AstExpression;
use crate::scanner::TokenKind;
use crate::value::JsValue;

#[derive(Debug, Clone, PartialEq)]
pub struct BinaryExpressionNode {
    pub left: Box<AstExpression>,
    pub operator: BinaryOperator,
    pub right: Box<AstExpression>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOperator {
    Add,
    Sub,
    Div,
    Mul,
    MulMul,
    LogicalOr,
    LogicalAnd,
    MoreThan,
    MoreThanOrEqual,
    LessThan,
    LessThanOrEqual,
    Equality,
    StrictEquality,
    Inequality,
    StrictInequality,
}

impl Execute for BinaryExpressionNode {
    fn execute(&self, interpreter: &Interpreter) -> Result<JsValue, String> {
        let evaluated_left_node = self.left.execute(interpreter)?;
        let evaluated_right_node = self.right.execute(interpreter)?;

        match self.operator {
            BinaryOperator::Add => &evaluated_left_node + &evaluated_right_node,
            BinaryOperator::Sub => &evaluated_left_node - &evaluated_right_node,
            BinaryOperator::Div => &evaluated_left_node / &evaluated_right_node,
            BinaryOperator::Mul => &evaluated_left_node * &evaluated_right_node,
            BinaryOperator::MulMul => {
                interpreter.exponentiation(&evaluated_left_node, &evaluated_right_node)
            }
            BinaryOperator::LogicalOr => {
                interpreter.logical_or(&evaluated_left_node, &evaluated_right_node)
            }
            BinaryOperator::LogicalAnd => {
                interpreter.logical_and(&evaluated_left_node, &evaluated_right_node)
            }
            BinaryOperator::MoreThan
            | BinaryOperator::MoreThanOrEqual
            | BinaryOperator::LessThan
            | BinaryOperator::LessThanOrEqual => {
                if let JsValue::Number(left_number) = evaluated_left_node {
                    if let JsValue::Number(right_number) = evaluated_right_node {
                        let value = match self.operator {
                            BinaryOperator::MoreThan => left_number > right_number,
                            BinaryOperator::MoreThanOrEqual => left_number >= right_number,
                            BinaryOperator::LessThan => left_number < right_number,
                            BinaryOperator::LessThanOrEqual => left_number <= right_number,
                            _ => unreachable!(),
                        };

                        return Ok(JsValue::Boolean(value));
                    }
                }

                Err(format!(
                    "Cannot compare value with type \"{}\" and \"{}\"",
                    evaluated_left_node.get_type_as_str(),
                    evaluated_right_node.get_type_as_str()
                ).to_string())
            }
            BinaryOperator::Equality
            | BinaryOperator::StrictEquality
            | BinaryOperator::Inequality
            | BinaryOperator::StrictInequality => {
                match (&evaluated_left_node, &evaluated_right_node) {
                    (JsValue::Number(left_number), JsValue::Number(right_number)) => {
                        let value = match self.operator {
                            BinaryOperator::Equality | BinaryOperator::StrictEquality => {
                                left_number == right_number
                            }
                            BinaryOperator::Inequality | BinaryOperator::StrictInequality => {
                                left_number != right_number
                            }
                            _ => unreachable!(),
                        };

                        return Ok(JsValue::Boolean(value));
                    },
                    (JsValue::String(left_string), JsValue::String(right_string)) => {
                        let value = match self.operator {
                            BinaryOperator::Equality | BinaryOperator::StrictEquality => {
                                left_string == right_string
                            }
                            BinaryOperator::Inequality | BinaryOperator::StrictInequality => {
                                left_string != right_string
                            }
                            _ => unreachable!(),
                        };

                        return Ok(JsValue::Boolean(value));
                    },
                    (JsValue::Object(object_left), JsValue::Object(object_right)) => {
                        let value = match self.operator {
                            BinaryOperator::Equality | BinaryOperator::StrictEquality => {
                                Rc::ptr_eq(object_left, object_right)
                            }
                            BinaryOperator::Inequality | BinaryOperator::StrictInequality => {
                                !Rc::ptr_eq(object_left, object_right)
                            }
                            _ => unreachable!(),
                        };

                        return Ok(JsValue::Boolean(value));
                    },
                    (JsValue::Boolean(boolean_left), JsValue::Boolean(boolean_right)) => Ok(JsValue::Boolean(boolean_left == boolean_right)),
                    _ => Ok(JsValue::Boolean(false))
                }
            }
        }
    }
}

impl TryFrom<&TokenKind> for BinaryOperator {
    type Error = String;

    fn try_from(value: &TokenKind) -> Result<Self, Self::Error> {
        match value {
            TokenKind::Plus => Ok(Self::Add),
            TokenKind::Minus => Ok(Self::Sub),
            TokenKind::Mul => Ok(Self::Mul),
            TokenKind::MulMul => Ok(Self::MulMul),
            TokenKind::Div => Ok(Self::Div),
            TokenKind::Or => Ok(Self::LogicalOr),
            TokenKind::And => Ok(Self::LogicalAnd),
            TokenKind::LessThan => Ok(Self::LessThan),
            TokenKind::LessThanOrEqual => Ok(Self::LessThanOrEqual),
            TokenKind::MoreThan => Ok(Self::MoreThan),
            TokenKind::MoreThanOrEqual => Ok(Self::MoreThanOrEqual),
            TokenKind::Equality => Ok(Self::Equality),
            TokenKind::StrictEquality => Ok(Self::StrictEquality),
            TokenKind::Inequality => Ok(Self::Inequality),
            TokenKind::StrictInequality => Ok(Self::StrictInequality),
            _ => Err("Cannot convert token kind to binary operator".to_string()),
        }
    }
}
