use std::{cell::RefCell, cell::RefMut, rc::Rc};

use crate::node::*;
mod environment;
use ariadne::{Label, Report, ReportKind, Source};
pub use environment::Environment;
mod js_value;
pub use js_value::*;
use std::collections::HashMap;
use std::fmt::format;

pub struct Interpreter {
    environment: Environment,
}

impl Interpreter {
    pub fn eval_node(&mut self, node: &NodeKind) -> Result<Option<JsValue>, String> {
        match node {
            NodeKind::StringLiteral(node) => Ok(Some(JsValue::String(node.value.to_string()))),
            NodeKind::NumberLiteral(value) => Ok(Some(JsValue::Number(*value))),
            NodeKind::NullLiteral => Ok(Some(JsValue::Null)),
            NodeKind::UndefinedLiteral => Ok(Some(JsValue::Undefined)),
            NodeKind::BooleanLiteral(value) => Ok(Some(JsValue::Boolean(*value))),
            NodeKind::VariableDeclaration(node) => match self.eval_variable_declaration(node) {
                Ok(_) => Ok(None),
                Err(error) => Err(error),
            },
            NodeKind::BinaryExpression(node) => match self.eval_binary_expression(node) {
                Ok(value) => Ok(Some(value)),
                Err(error) => Err(error),
            },
            NodeKind::ProgramStatement(node) => match self.eval_statements(&node.statements) {
                Ok(value) => Ok(value),
                Err(error) => Err(error),
            },
            NodeKind::BlockStatement(node) => match self.eval_block_statement(&node) {
                Ok(value) => Ok(value),
                Err(error) => Err(error),
            },
            NodeKind::Identifier(node) => Ok(Some(self.eval_identifier(node)?)),
            NodeKind::IfStatement(node) => {
                self.eval_if_statement(node)?;
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
                Ok(Some(self.eval_function_declaration(node)?))
            }
            NodeKind::FunctionExpression(node) => {
                Ok(Some(self.eval_function_expression(node).unwrap()))
            }
            NodeKind::CallExpression(node) => Ok(Some(self.eval_call_expression(node)?)),
            NodeKind::ForStatement(node) => {
                self.eval_for_statement(node);
                return Ok(None);
            }
            NodeKind::ReturnStatement(node) => self.eval_return_statement(node).map(|x| Some(x)),
            NodeKind::ConditionalExpression(node) => {
                return self.eval_conditional_expression(node).map(|x| Some(x));
            }
            NodeKind::ClassDeclaration(node) => self.eval_class_declaration(node).map(|x| Some(x)),
            NodeKind::ObjectExpression(node) => self.eval_object_expression(node).map(|x| Some(x)),
            NodeKind::MemberExpression(node) => self.eval_member_expression(node).map(|x| Some(x)),
            NodeKind::NewExpression(node) => self.eval_new_expression(node).map(|x| Some(x)),
            NodeKind::ThisExpression => Ok(self.eval_this_expression()),
            _ => todo!(),
        }
    }

    fn eval_this_expression(&mut self) -> Option<JsValue> {
        return self.environment.get_variable_value("this");
    }

    fn eval_new_expression(&mut self, node: &NewExpressionNode) -> Result<JsValue, String> {
        unimplemented!()
    }

    fn eval_block_statement(
        &mut self,
        node: &BlockStatementNode,
    ) -> Result<Option<JsValue>, String> {
        let block_environment = self.create_new_environment();
        self.environment = block_environment;
        let result = self.eval_statements(&node.statements);
        self.pop_environment();
        return result;
    }

    fn eval_member_expression(&mut self, node: &MemberExpressionNode) -> Result<JsValue, String> {
        let property_key = self.eval_member_expression_key(&node.property.node, node.computed)?;
        let resolved_object = self.eval_node(&node.object.node)?;

        if let Some(JsValue::Object(object)) = resolved_object {
            return Ok(object
                .borrow_mut()
                .get_value_property(property_key.as_str()));
        } else {
            return Err("Is not an object".to_string());
        }
    }

    fn eval_member_expression_key(
        &mut self,
        node: &NodeKind,
        computed: bool,
    ) -> Result<String, String> {
        if computed {
            let computed_key = self.eval_node(&node)?.unwrap();

            return match computed_key {
                JsValue::String(value) => Ok(value),
                JsValue::Number(value) => Ok(value.to_string()),
                _ => Err("".to_string()),
            };
        } else {
            return match node {
                NodeKind::StringLiteral(value) => Ok(value.value.clone()),
                NodeKind::NumberLiteral(value) => Ok(value.to_string()),
                NodeKind::Identifier(node) => Ok(node.id.clone()),
                _ => Err("Object key should be an identifier".to_string()),
            };
            //            if let NodeKind::Identifier(node) = &node {
            //                return Ok(node.id.clone());
            //            } else {
            //                return Err("Object key should be an identifier".to_string());
            //            }
        }
    }

    fn eval_object_expression(&mut self, node: &ObjectExpressionNode) -> Result<JsValue, String> {
        let mut object_value = JsObject::new_empty();

        for property in &node.properties {
            let key = self.eval_member_expression_key(&property.key.node, property.computed)?;
            object_value.add_property(&key, self.eval_node(&property.value.node)?.unwrap());
        }

        return Ok(JsValue::Object(Rc::new(RefCell::new(object_value))));
    }

    fn eval_object_property(&mut self, node: &ObjectPropertyNode) -> Result<JsValue, String> {
        return self.eval_node(&node.value.node).map(|x| x.unwrap());
    }

    fn eval_class_declaration(&mut self, node: &ClassDeclarationNode) -> Result<JsValue, String> {
        for class_method in &node.methods {}

        unimplemented!()
    }

    fn eval_conditional_expression(
        &mut self,
        node: &ConditionalExpressionNode,
    ) -> Result<JsValue, String> {
        let test = self.eval_node(&node.test.node)?.unwrap();

        let branch = if test.to_bool() {
            &node.consequent.node
        } else {
            &node.alternative.node
        };

        return self
            .eval_node(branch)
            .map(|x| x.unwrap_or(JsValue::Undefined));
    }

    fn eval_return_statement(&mut self, node: &ReturnStatementNode) -> Result<JsValue, String> {
        self.eval_node(&node.expression.node)
            .map(|x| x.unwrap_or(JsValue::Undefined))
    }

    fn eval_for_statement(&mut self, node: &ForStatementNode) {
        self.environment = self.create_new_environment();

        if node.init.is_some() {
            self.eval_node(&node.init.as_ref().unwrap().node).unwrap();
        }

        while self
            .eval_node(&node.test.as_ref().unwrap().node)
            .unwrap()
            .unwrap()
            .to_bool()
        {
            self.eval_node(&node.body.as_ref().node).unwrap();
            self.eval_node(&node.update.as_ref().unwrap().node)
                .unwrap()
                .unwrap();
        }

        self.pop_environment();
    }

    fn eval_call_expression(&mut self, node: &CallExpressionNode) -> Result<JsValue, String> {
        let callee = self.eval_node(&node.callee.node)?;

        if let Some(JsValue::Function(function)) = &callee {
            let mut function_execution_environment = self.create_new_environment();

            if let NodeKind::MemberExpression(expr) = &node.callee.node {
                function_execution_environment.define_variable("this".to_string(), self.eval_node(&expr.object.node).unwrap().unwrap());
                // function_execution_environment.print_variables();
            }

            let values: Vec<JsValue> = node.params.iter().map(|param| {
                return self.eval_node(&param.node).unwrap().unwrap_or(JsValue::Undefined);
            }).collect();

            match function {
                Func::Js(function) => {
                    function
                        .arguments
                        .iter()
                        .zip(&node.params)
                        .for_each(|(arg, node)| {
                            let value = self.eval_node(&node.node)
                                .unwrap()
                                .unwrap_or(JsValue::Undefined);

                            function_execution_environment
                                .define_variable(
                                    arg.name.clone(),
                                    value,
                                )
                                .unwrap();
                        });
                    self.environment = function_execution_environment;
                    let result = function.call(self, &values);
                    self.pop_environment();
                    return result;
                }
                Func::Native(function) => {
                    self.environment = function_execution_environment;
                    let result = function.call(self, &values);
                    self.pop_environment();
                    return result;
                }
            }
        } else {
            return Err(format!(
                "{} is not callable",
                callee.unwrap().get_type_as_str()
            ));
        }
    }

    fn create_new_environment(&self) -> Environment {
        return Environment::new(Rc::new(RefCell::new(self.environment.clone())));
    }

    fn pop_environment(&mut self) {
        self.environment = self.environment.get_parent().unwrap().borrow().clone();
    }

    fn eval_function_expression(&mut self, node: &FunctionExpressionNode) -> Result<JsValue, String> {
        return Ok(self.create_js_function(&node.arguments, *node.body.clone()));
    }

    fn eval_function_declaration(
        &mut self,
        node: &FunctionDeclarationNode,
    ) -> Result<JsValue, String> {
        let js_function_value = self.create_js_function(&node.arguments, *node.body.clone());
        self.environment.define_variable(node.name.id.clone(), js_function_value.clone())?;
        return Ok(js_function_value);
    }

    fn create_js_function(&mut self, function_arguments: &Vec<FunctionArgument>, body: Node) -> JsValue {
        let mut arguments = vec![];

        for fn_arg_node in function_arguments {
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

        return JsValue::Function(Func::Js(JsFunction {
            arguments,
            environment: Box::new(self.environment.clone()),
            body: Box::new(body.node.clone()),
        }));
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
                        let original_value =
                            self.environment.get_variable_value(&id_node.id).unwrap();
                        self.div(&original_value, &right_hand_value)
                    }
                    AssignmentOperator::MulEqual => {
                        let original_value =
                            self.environment.get_variable_value(&id_node.id).unwrap();
                        self.mul(&original_value, &right_hand_value)
                    }
                    AssignmentOperator::ExponentiationEqual => {
                        let original_value =
                            self.environment.get_variable_value(&id_node.id).unwrap();
                        self.exponentiation(&original_value, &right_hand_value)
                    }
                    AssignmentOperator::Equal => Ok(right_hand_value),
                }
                .unwrap();

                self.environment.assign_variable(id_node.id.clone(), new_variable_value.clone())?;
                return Ok(new_variable_value);
            }
            NodeKind::MemberExpression(node) => {
                let object = self.eval_node(&node.object.node)?.unwrap();

                if let JsValue::Object(object_value) = object {
                    let mut object = object_value;

                    let key =
                        self.eval_member_expression_key(&node.property.node, node.computed)?;

                    object
                        .borrow_mut()
                        .add_property(key.as_str(), right_hand_value);

                    return Ok(JsValue::Object(object));
                } else {
                    return Err(
                        "Cannot assign: left hand side expression is not an object".to_string()
                    );
                }
            }
            _ => todo!(),
        }
    }

    fn get_member_expression_key(&mut self, node: &Node) -> Result<String, String> {
        match &node.node {
            NodeKind::Identifier(node) => Ok(node.id.clone()),
            node => {
                let evaluated_node = self.eval_node(&node)?.unwrap();

                match evaluated_node {
                    JsValue::String(value) => Ok(value),
                    JsValue::Number(value) => Ok(value.to_string()),
                    value => Err(format!(
                        "Type {} cannot be used as an object key",
                        value.get_type_as_str()
                    )),
                }
            }
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
        // println!("eval_identifier {:#?}", node);
        return self
            .environment
            .get_variable_value(&node.id)
            .ok_or(format!("Variable \"{}\" not found", node.id));
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
            BinaryOperator::MulMul => {
                self.exponentiation(&evaluated_left_node, &evaluated_right_node)
            }
            BinaryOperator::LogicalOr => {
                self.logical_or(&evaluated_left_node, &evaluated_right_node)
            }
            BinaryOperator::LogicalAnd => {
                self.logical_and(&evaluated_left_node, &evaluated_right_node)
            }
            BinaryOperator::MoreThan
            | BinaryOperator::MoreThanOrEqual
            | BinaryOperator::LessThan
            | BinaryOperator::LessThanOrEqual => {
                if let JsValue::Number(left_number) = evaluated_left_node {
                    if let JsValue::Number(right_number) = evaluated_right_node {
                        let value = match node.operator {
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
                )
                .to_string())
            }
            BinaryOperator::Equality
            | BinaryOperator::StrictEquality
            | BinaryOperator::Inequality
            | BinaryOperator::StrictInequality => {
                if let JsValue::Number(left_number) = evaluated_left_node {
                    if let JsValue::Number(right_number) = evaluated_right_node {
                        let value = match node.operator {
                            BinaryOperator::Equality | BinaryOperator::StrictEquality => {
                                left_number == right_number
                            }
                            BinaryOperator::Inequality | BinaryOperator::StrictInequality => {
                                left_number != right_number
                            }
                            _ => unreachable!(),
                        };

                        return Ok(JsValue::Boolean(value));
                    }
                }

                if let JsValue::Object(object_left) = &evaluated_left_node {
                    if let JsValue::Object(object_right) = &evaluated_right_node {
                        let value = match node.operator {
                            BinaryOperator::Equality | BinaryOperator::StrictEquality => {
                                object_left == object_right
                            }
                            BinaryOperator::Inequality | BinaryOperator::StrictInequality => {
                                object_left != object_right
                            }
                            _ => unreachable!(),
                        };

                        return Ok(JsValue::Boolean(value));
                    }
                }

                Err(format!(
                    "Cannot compare value with type \"{}\" and \"{}\"",
                    evaluated_left_node.get_type_as_str(),
                    evaluated_right_node.get_type_as_str()
                )
                .to_string())
            }
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

    fn eval_statements(&mut self, statements: &Vec<Node>) -> Result<Option<JsValue>, String> {
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

    fn exponentiation(&self, left: &JsValue, right: &JsValue) -> Result<JsValue, String> {
        match (left, right) {
            (JsValue::Number(left_number), JsValue::Number(right_number)) => {
                Ok(JsValue::Number(left_number.powf(*right_number)))
            }
            _ => Err(format!(
                "exponentiation of types '{}' and '{}' is not possible",
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
            (JsValue::String(left_string), JsValue::Number(right_number)) => {
                let mut result_string = left_string.clone();
                result_string.push_str(right_number.to_string().as_str());
                return Ok(JsValue::String(result_string));
            }
            (JsValue::Number(left_number), JsValue::Number(right_number)) => {
                Ok(JsValue::Number(left_number + right_number))
            }
            _ => Err(format!(
                "addition of types '{}' and '{}' is not possible",
                left.get_type_as_str(),
                right.get_type_as_str()
            )),
        }
    }
}

fn get_global_environment() -> Environment {
    fn console_log(interpreter: &mut Interpreter, arguments: &Vec<JsValue>) -> Result<JsValue, String> {
        let result = arguments.iter()
            .map(|arg| format!("{}", arg))
            .collect::<Vec<String>>()
            .join(" ");
        println!("{result}");
        return Ok(JsValue::Undefined);
    }

    Environment::new_with_variables([
        ("console".to_string(), create_js_object(JsObject {
            properties: HashMap::from([
                ("log".to_string(), create_native_function(console_log))
            ]),
            prototype: None
        }))
    ])
}

impl Default for Interpreter {
    fn default() -> Self {
        let env = get_global_environment();
        Self { environment: env }
    }
}

trait Visitor {
    fn visit(&self) -> Box<dyn Iterator<Item = &NodeKind> + '_>;
}

impl Visitor for Node {
    fn visit(&self) -> Box<dyn Iterator<Item = &NodeKind> + '_> {
//        println!("visit {:?}", self.node);
        match &self.node {
            NodeKind::ProgramStatement(node) => Box::new(
                node.statements
                    .iter()
                    .map(|x| {
                        x.visit()
                    })
                    .flatten(),
            ),
            NodeKind::StringLiteral(_)
            | NodeKind::NumberLiteral(_)
            | NodeKind::BooleanLiteral(_)
            | NodeKind::NullLiteral
            | NodeKind::UndefinedLiteral
            | NodeKind::ThisExpression
            | NodeKind::Identifier(_) => Box::new(iter::once(&self.node).into_iter()),
            NodeKind::BinaryExpression(node) => Box::new(
                iter::once(&node.left.as_ref().node).chain(iter::once(&node.right.as_ref().node)),
            ),
            NodeKind::VariableDeclaration(_) => todo!(),
            NodeKind::AssignmentExpression(_) => todo!(),
            NodeKind::BlockStatement(node) => {
                return Box::new(
                    node.statements
                    .iter()
                    .map(|x| {
                        x.visit()
                    })
                    .flatten(),
            )
            },
            NodeKind::IfStatement(node) => {
                //                let b: impl Iterator<Item = &NodeKind> + '_ = iter::empty::<&NodeKind>().into();

//                let a = &node
//                    .else_branch
//                    .map(|x| x.visit());
//
//                let b = if let Some(n) = a {
//                    n
//                } else {
//                    Box::new(iter::empty())
//                }.into_iter();

//                let c = iter::once(&node.condition.node).chain(b);

//                let a = node.condition.visit().chain(iter::empty()).into_iter();
//                let mut a = node.then_branch.visit();
//
//                let b = a.chain(Box::new(vec![].into_iter()));
//                return b.into_iter();
                unimplemented!()

//                return b;
//                return node.then_branch.visit();

            }
            NodeKind::WhileStatement(_) => todo!(),
            NodeKind::ForStatement(_) => todo!(),
            NodeKind::FunctionDeclaration(_) => todo!(),
            NodeKind::ReturnStatement(_) => todo!(),
            NodeKind::CallExpression(_) => todo!(),
            NodeKind::ConditionalExpression(_) => todo!(),
            NodeKind::MemberExpression(_) => todo!(),
            NodeKind::ClassDeclaration(_) => todo!(),
            NodeKind::NewExpression(_) => todo!(),
            NodeKind::ObjectProperty(_) => todo!(),
            NodeKind::ObjectExpression(_) => todo!(),
            NodeKind::FunctionExpression(_) => todo!(),
        }
    }
}

impl Visitor for NodeKind {
    fn visit(&self) -> Box<dyn Iterator<Item = &NodeKind> + '_> {
        match self {
            NodeKind::ProgramStatement(node) => node.visit(),
            NodeKind::StringLiteral(_) => todo!(),
            NodeKind::NumberLiteral(_) => todo!(),
            NodeKind::BooleanLiteral(_) => todo!(),
            NodeKind::NullLiteral => todo!(),
            NodeKind::UndefinedLiteral => todo!(),
            NodeKind::ThisExpression => todo!(),
            NodeKind::Identifier(_) => todo!(),
            NodeKind::BinaryExpression(_) => todo!(),
            NodeKind::VariableDeclaration(_) => todo!(),
            NodeKind::AssignmentExpression(_) => todo!(),
            NodeKind::BlockStatement(_) => todo!(),
            NodeKind::IfStatement(_) => todo!(),
            NodeKind::WhileStatement(_) => todo!(),
            NodeKind::ForStatement(_) => todo!(),
            NodeKind::FunctionDeclaration(_) => todo!(),
            NodeKind::ReturnStatement(_) => todo!(),
            NodeKind::CallExpression(_) => todo!(),
            NodeKind::ConditionalExpression(_) => todo!(),
            NodeKind::MemberExpression(_) => todo!(),
            NodeKind::ClassDeclaration(_) => todo!(),
            NodeKind::NewExpression(_) => todo!(),
            NodeKind::ObjectProperty(_) => todo!(),
            NodeKind::ObjectExpression(_) => todo!(),
            NodeKind::FunctionExpression(_) => todo!(),
        }
    }
}

use std::iter;
use crate::interpreter::js_value::NativeFunction;
use crate::interpreter::JsValue::Function;

impl Visitor for IfStatementNode {
    fn visit(&self) -> Box<dyn Iterator<Item = &NodeKind> + '_> {
        //        let a = self.condition.node.visit();
        //        let a: dyn Iterator<Item = &NodeKind> + '_ = &self.else_branch.map_or(iter::empty::<_>().into_iter(), |x| x.node.visit().as_ref());

        let c: Box<dyn Iterator<Item = &NodeKind> + '_> = Box::new(
            self.condition
                .node
                .visit()
                .chain(self.then_branch.node.visit().chain(iter::empty())),
        );
        return c;
        //        todo!()
    }
}

//impl Visitor for StringLiteralNode {
//    fn visit(&self) ->Box<dyn Iterator<Item = &NodeKind> + '_> {
//        return Box::new(iter::once(NodeKind::StringLiteral(self.clone())).into_iter().into_iter());
//
//        todo!()
//    }
//}

//impl Visitor for f64 {
//    fn visit(&self) ->Box<dyn Iterator<Item = &NodeKind> + '_> {
//        Box::new(iter::once(&self).into_iter())
//    }
//}

//impl Visitor for bool {
//    fn visit(&self) ->Box<dyn Iterator<Item = &NodeKind> + '_> {
//        Box::new(iter::once(&NodeKind::BooleanLiteral(*self)).into_iter())
//    }
//}

impl Visitor for ProgramNode {
    fn visit(&self) -> Box<dyn Iterator<Item = &NodeKind> + '_> {
        Box::new(self.statements.iter().map(|x| x.node.visit()).flatten())
    }
}

//impl Visitor for ProgramNode {
//    fn visit(&self) {
//        self.statements.iter().for_each(|node| node.node.visit());
//    }
//}

//impl Visitor for IdentifierNode {
//    fn visit(&self) {
//        self
//    }
//}

pub fn walk(node: &Node) {
    node.visit().for_each(|x| println!("{:?}", x));
}

enum NodeTest {
    A(A),
    B(B),
    C(Vec<B>),
}

struct A {
    b: Box<NodeTest>,
}

struct B {
    c: Box<NodeTest>,
    d: Box<NodeTest>,
}

//impl Iterator for A {
//    type Item = NodeTest;
//
//    fn next(&mut self) -> Option<Self::Item> {
//        let a = self.b.as_ref().into_iter();
//
////        self.b.as_ref().into_iter()
//    }
//}

impl Iterator for &NodeTest {
    type Item = NodeTest;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}
