use std::rc::Rc;

use crate::node::*;
mod environment;
use ariadne::{Label, Report, ReportKind, Source};
pub use environment::Environment;
mod js_value;
pub use js_value::{JsFunction, JsFunctionArg, JsValue};

pub struct Interpreter {
    environment: Environment,
    //    env_stack: Vec<Environment>
}

impl Interpreter {
    pub fn eval_node(&mut self, node: &NodeKind) -> Result<Option<JsValue>, String> {
//        println!("eval node: {:#?}", node);
        match node {
            NodeKind::StringLiteral(value) => Ok(Some(JsValue::String(value.to_string()))),
            NodeKind::NumberLiteral(value) => Ok(Some(JsValue::Number(*value))),
            NodeKind::NullLiteral => Ok(Some(JsValue::Null)),
            NodeKind::UndefinedLiteral => Ok(Some(JsValue::Undefined)),
            NodeKind::BooleanLiteral(value) => Ok(Some(JsValue::Boolean(*value))),
            NodeKind::VariableDeclaration(node) => match self.eval_variable_declaration(node) {
                Ok(_) => Ok(None),
                Err(error) => Err(error.clone()),
            },
            NodeKind::BinaryExpression(node) => match self.eval_binary_expression(node) {
                Ok(value) => Ok(Some(value)),
                Err(error) => Err(error),
            },
            NodeKind::ProgramStatement(node) => match self.eval_statements(&node.statements) {
                Ok(value) => Ok(value),
                Err(error) => Err(error),
            },
            NodeKind::BlockStatement(node) => match self.eval_statements(&node.statements) {
                Ok(value) => Ok(value),
                Err(error) => Err(error),
            },
            NodeKind::Identifier(node) => Ok(Some(self.eval_identifier(node).unwrap())),
            NodeKind::IfStatement(node) => {
                self.eval_if_statement(node).unwrap();
                return Ok(None);
            }
            NodeKind::PrintStatement(node) => {
                self.eval_print_statement(node);
                return Ok(None);
            }
            NodeKind::AssignmentExpression(node) => {
                return self
                    .eval_assignment_expression(node)
                    .map(|value| Some(value));
            }
            NodeKind::WhileStatement(node) => {
                self.eval_while_statement(node);
                return Ok(None);
            }
            NodeKind::FunctionDeclaration(node) => {
                Ok(Some(self.eval_function_declaration(node).unwrap()))
            }
            NodeKind::CallExpression(node) => Ok(Some(self.eval_call_expression(node).unwrap())),
            NodeKind::ForStatement(node) => {
                self.eval_for_statement(node);
                return Ok(None);
            },
            NodeKind::ReturnStatement(node) => self.eval_return_statement(node).map(|x| Some(x)),
            NodeKind::ConditionalExpression(node) => self.eval_conditional_expression(node).map(|x| Some(x)),
            _ => todo!(),
        }
    }

    fn eval_conditional_expression(&mut self, node: &ConditionalExpressionNode) -> Result<JsValue, String> {
        let test = self.eval_node(&node.test.node)?.unwrap();

        if test.to_bool() {
            return self.eval_node(&node.consequent.node).map(|x| x.unwrap_or(JsValue::Undefined));
        } else {
            return self.eval_node(&node.alternative.node).map(|x| x.unwrap_or(JsValue::Undefined));
        }
    }

    fn eval_return_statement(&mut self, node: &ReturnStatementNode) -> Result<JsValue, String> {
        self.eval_node(&node.expression.node).map(|x| x.unwrap_or(JsValue::Undefined))
    }

    fn eval_for_statement(&mut self, node: &ForStatementNode) {
        println!("eval_for_statement: {:?}", node);
        if node.init.is_some() {
            self.eval_node(&node.init.as_ref().unwrap().node).unwrap();
        }

        while self.eval_node(&node.test.as_ref().unwrap().node).unwrap().unwrap().to_bool() {
            self.eval_node(&node.body.as_ref().node).unwrap();
            self.eval_node(&node.update.as_ref().unwrap().node).unwrap().unwrap();
        }
    }

    fn eval_call_expression(&mut self, node: &CallExpressionNode) -> Result<JsValue, String> {
        let callee = self.eval_node(&node.callee.node)?;

        if let Some(JsValue::Function(function)) = &callee {
            let mut function_execution_environment = Environment::new(Box::new(self.environment.clone()));
            function.arguments.iter().zip(&node.params).for_each(|(arg, node)| {
                function_execution_environment.define_variable(arg.name.clone(), self.eval_node(&node.node).unwrap().unwrap_or(JsValue::Undefined));
            });
            self.environment = function_execution_environment;
            let result = self.eval_node(function.body.as_ref());
            self.environment = self.environment.get_parent().unwrap();
            return result.map(|x| x.unwrap_or(JsValue::Undefined));
        } else {
            return Err(format!(
                "{} is not callable",
                callee.unwrap().get_type_as_str()
            ));
        }
    }

    fn eval_function_declaration(
        &mut self,
        node: &FunctionDeclarationNode,
    ) -> Result<JsValue, String> {
        let mut arguments = vec![];

        for fn_arg_node in &node.arguments {
            let default_value = fn_arg_node
                .default_value
                .as_ref()
                .map(|node| self.eval_node(&node.as_ref().node).unwrap())
                .flatten()
                .unwrap_or(JsValue::Undefined);

            arguments.push(JsFunctionArg {
                name: fn_arg_node.name.clone(),
                default_value,
            });
        }

        let js_function_value = JsValue::Function(JsFunction {
            name: node.name.id.clone(),
            arguments,
            environment: Box::new(self.environment.clone()),
            body: Box::new(node.body.node.clone()),
        });

        println!("{:?}", js_function_value);

        self.environment
            .define_variable(node.name.id.clone(), js_function_value.clone())?;

        return Ok(js_function_value);
    }

    fn eval_assignment_expression(
        &mut self,
        node: &AssignmentExpressionNode,
    ) -> Result<JsValue, String> {
        let right_hand_value = self.eval_node(&node.right.node).unwrap().unwrap();

        match &node.left.as_ref().node {
            NodeKind::Identifier(id_node) => {
                let new_variable_value = match node.operator {
                    AssignmentOperator::AddEqual => {
                        let original_value =
                            self.environment.get_variable_value(&id_node.id).unwrap();
                        self.add(&original_value, &right_hand_value)
                    }
                    AssignmentOperator::SubEqual => {
                        let original_value =
                            self.environment.get_variable_value(&id_node.id).unwrap();
                        self.sub(&original_value, &right_hand_value)
                    }
                    AssignmentOperator::DivEqual => {
                        let original_value = self.environment.get_variable_value(&id_node.id).unwrap();
                        self.div(&original_value, &right_hand_value)
                    }
                    AssignmentOperator::MulEqual => {
                        let original_value =
                            self.environment.get_variable_value(&id_node.id).unwrap();
                        self.mul(&original_value, &right_hand_value)
                    }
                    AssignmentOperator::Equal => Ok(right_hand_value),
                }
                .unwrap();

                self.environment
                    .assign_variable(id_node.id.clone(), new_variable_value.clone())
                    .unwrap();
                return Ok(new_variable_value);
            }
            _ => todo!(),
        }
    }

    fn eval_while_statement(&mut self, node: &WhileStatementNode) {
        while self
            .eval_node(&node.condition.node)
            .unwrap()
            .unwrap()
            .to_bool()
        {
            self.eval_node(&node.body.as_ref().node).unwrap();
        }
    }

    fn eval_print_statement(&mut self, node: &PrintStatementNode) {
        let result = self.eval_node(&node.expression.node).unwrap();

        match result {
            Some(value) => println!("{}", value),
            None => println!("{}", JsValue::Undefined),
        }
    }

    fn eval_if_statement(&mut self, node: &IfStatementNode) -> Result<(), String> {
        let condition_value = self
            .eval_node(&node.condition.node)
            .expect("Error during evaluation of condition of 'if statement'")
            .unwrap_or(JsValue::Undefined);

        if condition_value.to_bool() {
            self.eval_node(&node.then_branch.as_ref().node).unwrap();
            return Ok(());
        } else if let Some(node) = node.else_branch.as_ref() {
            self.eval_node(&node.node).unwrap();
            return Ok(());
        }

        return Ok(());
    }

    fn eval_identifier(&self, node: &IdentifierNode) -> Result<JsValue, String> {
        return self
            .environment
            .get_variable_value(&node.id)
            .ok_or("Variable not found".to_string());
    }

    fn eval_variable_declaration(&mut self, node: &VariableDeclarationNode) -> Result<(), String> {
        let value = if let Some(value) = &node.value {
            self.eval_node(&value.as_ref().node)
                .expect("Error during variable value evaluation")
                .expect("No value")
        } else {
            JsValue::Undefined
        };
        return self.environment.define_variable(node.id.clone(), value);
    }

    fn eval_binary_expression(&mut self, node: &BinaryExpressionNode) -> Result<JsValue, String> {
        let evaluated_left_node = self
            .eval_node(&node.left.node)
            .expect("Left expression evaluation error")
            .expect("Left expression has no value");
        let evaluated_right_node = self
            .eval_node(&node.right.node)
            .expect("Right expression evaluation error")
            .expect("Right expression has no value");

        match node.operator {
            BinaryOperator::Add => self.add(&evaluated_left_node, &evaluated_right_node),
            BinaryOperator::Sub => self.sub(&evaluated_left_node, &evaluated_right_node),
            BinaryOperator::Div => self.div(&evaluated_left_node, &evaluated_right_node),
            BinaryOperator::Mul => self
                .mul(&evaluated_left_node, &evaluated_right_node)
                .map_err(|_e| format!("")),
            BinaryOperator::LogicalOr => {
                self.logical_or(&evaluated_left_node, &evaluated_right_node)
            }
            BinaryOperator::LogicalAnd => self.logical_and(&evaluated_left_node, &evaluated_right_node),
            BinaryOperator::MoreThan | BinaryOperator::MoreThanOrEqual | BinaryOperator::LessThan | BinaryOperator::LessThanOrEqual => {
                if let JsValue::Number(left_number) = evaluated_left_node {
                    if let JsValue::Number(right_number) = evaluated_right_node {
                        let value = match node.operator {
                            BinaryOperator::MoreThan => left_number > right_number,
                            BinaryOperator::MoreThanOrEqual => left_number >= right_number,
                            BinaryOperator::LessThan => left_number < right_number,
                            BinaryOperator::LessThanOrEqual => left_number <= right_number,
                            _ => unreachable!()
                        };

                        return Ok(JsValue::Boolean(value));
                    }
                }

                Err(format!("Cannot compare value with type \"{}\" and \"{}\"", evaluated_left_node.get_type_as_str(), evaluated_right_node.get_type_as_str()).to_string())
            },
            BinaryOperator::Equality | BinaryOperator::StrictEquality | BinaryOperator::Inequality | BinaryOperator::StrictInequality => {
                if let JsValue::Number(left_number) = evaluated_left_node {
                    if let JsValue::Number(right_number) = evaluated_right_node {
                        let value = match node.operator {
                            BinaryOperator::Equality | BinaryOperator::StrictEquality => left_number == right_number,
                            BinaryOperator::Inequality | BinaryOperator::StrictInequality => left_number != right_number,
                            _ => unreachable!()
                        };

                        return Ok(JsValue::Boolean(value));
                    }
                }

                Err(format!("Cannot compare value with type \"{}\" and \"{}\"", evaluated_left_node.get_type_as_str(), evaluated_right_node.get_type_as_str()).to_string())
            },
        }
    }

    fn logical_or(&mut self, left: &JsValue, right: &JsValue) -> Result<JsValue, String> {
        if left.to_bool() {
            return Ok(left.clone());
        }
        return Ok(right.clone());
    }

    fn logical_and(&mut self, left: &JsValue, right: &JsValue) -> Result<JsValue, String> {
        if !left.to_bool() {
            return Ok(left.clone());
        }
        return Ok(right.clone());
    }

    fn eval_statements(
        &mut self,
        statements: &Vec<Node>,
    ) -> Result<Option<JsValue>, String> {
        let mut result: Option<JsValue> = None;

        for statement in statements {
            result = self.eval_node(&statement.node)?;
        }

        return Ok(result);
    }

    fn div(&self, left: &JsValue, right: &JsValue) -> Result<JsValue, String> {
        match (left, right) {
            (JsValue::Number(left_number), JsValue::Number(right_number)) => {
                Ok(JsValue::Number(left_number / right_number))
            }
            _ => Err(format!(
                "division of types '{}' and '{}' is not possible",
                left.get_type_as_str(),
                right.get_type_as_str()
            )),
        }
    }

    fn mul(&self, left: &JsValue, right: &JsValue) -> Result<JsValue, String> {
        match (left, right) {
            (JsValue::String(string), JsValue::Number(number)) => {
                Ok(JsValue::String(string.repeat(*number as usize)))
            }
            (JsValue::Number(left_number), JsValue::Number(right_number)) => {
                Ok(JsValue::Number(left_number * right_number))
            }
            _ => Err(format!(
                "multiplication of types '{}' and '{}' is not possible",
                left.get_type_as_str(),
                right.get_type_as_str()
            )),
        }
    }

    fn sub(&self, left: &JsValue, right: &JsValue) -> Result<JsValue, String> {
        match (left, right) {
            (JsValue::Number(left_number), JsValue::Number(right_number)) => {
                Ok(JsValue::Number(left_number - right_number))
            }
            _ => Err(format!(
                "subtraction of types '{}' and '{}' is not possible",
                left.get_type_as_str(),
                right.get_type_as_str()
            )),
        }
    }

    fn add(&self, left: &JsValue, right: &JsValue) -> Result<JsValue, String> {
        match (left, right) {
            (JsValue::String(left_string), JsValue::String(right_string)) => {
                let mut result_string = left_string.clone();
                result_string.push_str(right_string.as_str());
                return Ok(JsValue::String(result_string));
            }
            (JsValue::Number(left_number), JsValue::Number(right_number)) => {
                Ok(JsValue::Number(left_number + right_number))
            }
            _ => {
                Err(format!(
                    "addition of types '{}' and '{}' is not possible",
                    left.get_type_as_str(),
                    right.get_type_as_str()
                ))
            }
        }
    }
}

impl Default for Interpreter {
    fn default() -> Self {
        let env = Environment::default();
        Self {
            environment: env,
            //            env_stack: vec![env]
        }
    }
}
