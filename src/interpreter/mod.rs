use std::{cell::RefCell, rc::Rc};
use crate::node::*;
mod environment;
pub use environment::{Environment, EnvironmentRef};
use crate::value::{JsValue};
use crate::value::function::{JsFunction, JsFunctionArg, Callable};
use crate::value::object::{JsObject, ObjectKind};

const CONSTRUCTOR_METHOD_NAME: &'static str = "constructor";

pub struct Interpreter {
    environment: RefCell<EnvironmentRef>,
}

impl Interpreter {
    pub fn eval_node(&self, node: &AstStatement) -> Result<Option<JsValue>, String> {
        match node {
            AstStatement::VariableDeclaration(node) => match self.eval_variable_declaration(node) {
                Ok(_) => Ok(None),
                Err(error) => Err(error),
            },
            AstStatement::ProgramStatement(node) => match self.eval_statements(&node.statements) {
                Ok(value) => Ok(value),
                Err(error) => Err(error),
            },
            AstStatement::BlockStatement(node) => match self.eval_block_statement(&node) {
                Ok(value) => Ok(value),
                Err(error) => Err(error),
            },
            AstStatement::ExpressionStatement(node) => Ok(Some(self.eval_expression(node))),
            AstStatement::WhileStatement(node) => {
                self.eval_while_statement(node);
                return Ok(None);
            }
            AstStatement::FunctionDeclaration(node) => Ok(Some(self.eval_function_declaration(node)?)),
            AstStatement::ForStatement(node) => {
                self.eval_for_statement(node);
                return Ok(None);
            }
            AstStatement::ReturnStatement(node) => Ok(Some(self.eval_return_statement(node))),
            AstStatement::IfStatement(node) => {
                self.eval_if_statement(node)?;
                return Ok(None);
            },
            AstStatement::BreakStatement(_) => todo!()
        }
    }

    fn eval_expression(&self, expression: &AstExpression) -> JsValue {
        let value = match expression {
            AstExpression::Identifier(node) => Ok(self.eval_identifier(node)),
            AstExpression::AssignmentExpression(node) => self.eval_assignment_expression(node),
            AstExpression::ClassDeclaration(node) => self.eval_class_declaration(node),
            AstExpression::ObjectExpression(node) => self.eval_object_expression(node),
            AstExpression::MemberExpression(node) => self.eval_member_expression(node),
            AstExpression::NewExpression(node) => self.eval_new_expression(node),
            AstExpression::ThisExpression(_) => Ok(self.eval_this_expression()),
            AstExpression::ConditionalExpression(node) => Ok(self.eval_conditional_expression(node)),
            AstExpression::FunctionExpression(node) => self.eval_function_expression(node),
            AstExpression::CallExpression(node) => self.eval_call_expression(node),
            AstExpression::StringLiteral(node) => Ok(JsValue::String(node.value.clone())),
            AstExpression::NumberLiteral(node) => Ok(JsValue::Number(node.value)),
            AstExpression::NullLiteral(_) => Ok(JsValue::Null),
            AstExpression::UndefinedLiteral(_) => Ok(JsValue::Undefined),
            AstExpression::BooleanLiteral(node) => Ok(JsValue::Boolean(node.value)),
            AstExpression::BinaryExpression(node) => self.eval_binary_expression(node),
            AstExpression::ArrayExpression(node) => self.eval_array_expression(node),
        };

        value.unwrap()
    }

    fn eval_array_expression(&self, node: &ArrayExpressionNode) -> Result<JsValue, String> {
        let array_items: Vec<JsValue> = node.items.iter().map(|x| self.eval_expression(x)).collect();
        return Ok(JsObject::array(array_items).to_js_value());
    }

    fn eval_this_expression(&self) -> JsValue {
        return self.environment.borrow().borrow().get_context();
    }

    fn eval_new_expression(&self, node: &NewExpressionNode) -> Result<JsValue, String> {
        let callee = self.eval_expression(node.callee.as_ref());

        if !callee.is_function() {
            return Err("Cannot call non-function value".to_string());
        }

        let result = self.call_function(node.callee.as_ref(), node.arguments.as_ref(), true);

        result
    }

    fn eval_block_statement(&self, node: &BlockStatementNode) -> Result<Option<JsValue>, String> {
        let env = self.create_new_environment();
        self.set_environment(env);
        let result = self.eval_statements(&node.statements);
        self.pop_environment();
        return result;
    }

    fn eval_member_expression(&self, node: &MemberExpressionNode) -> Result<JsValue, String> {
        let property_key = self.eval_member_expression_key(&node.property, node.computed)?;
        let resolved_object = self.eval_expression(&node.object);

        match resolved_object {
            JsValue::Object(object) => {
                Ok(object.borrow_mut().get_property_value(property_key.as_str()))
            },
            _ => Err("Is not an object".to_string())
        }
    }

    fn eval_member_expression_key(
        &self,
        node: &AstExpression,
        computed: bool,
    ) -> Result<String, String> {
        if computed {
            let computed_key = self.eval_expression(&node);

            return match computed_key {
                JsValue::String(value) => Ok(value),
                JsValue::Number(value) => Ok(value.to_string()),
                _ => Err("".to_string()),
            };
        } else {
            return match node {
                AstExpression::StringLiteral(value) => Ok(value.value.clone()),
                AstExpression::NumberLiteral(node) => Ok(node.value.to_string()),
                AstExpression::Identifier(node) => Ok(node.id.clone()),
                _ => Err("Object key should be an identifier".to_string()),
            };
        }
    }

    fn eval_object_expression(&self, node: &ObjectExpressionNode) -> Result<JsValue, String> {
        let mut object_value = JsObject::empty();

        for property in &node.properties {
            let key = self.eval_member_expression_key(&property.key, property.computed)?;
            object_value.add_property(&key, self.eval_expression(&property.value));
        }

        return Ok(object_value.into());
    }

    /// In js a class is a function that gets `this` as variable when called with new keyword
    /// To do this we need:
    ///     1. construct a function from a constructor method
    ///     2. construct an object with class methods (prototype) excluding a "constructor" method
    ///
    /// For example:
    /// class A {
    ///   some_field = 5;
    ///
    ///   constructor(a, b = 10) {
    ///     this.a = a;
    ///     this.b = b;
    ///   }
    ///
    ///   getA() {
    ///     return this.a;
    ///   }
    ///
    ///   setB(newValue) {
    ///     this.b = newValue;
    ///   }
    /// }
    ///
    fn eval_class_declaration(&self, node: &ClassDeclarationNode) -> Result<JsValue, String> {
        let prototype_object = self.build_prototype_object_from_class_declaration(node);
        let mut constructor_function = self.build_constructor_from_class_declaration(node).to_object();

        constructor_function.set_prototype(prototype_object.to_ref());

        let constructor_function = JsValue::Object(constructor_function.to_ref());

        self.environment.borrow().borrow_mut().define_variable(
            node.name.id.clone(),
            constructor_function.clone(),
            false
        ).unwrap();

        Ok(constructor_function)
    }

    fn build_constructor_from_class_declaration(&self, node: &ClassDeclarationNode) -> JsFunction {
        let constructor_method = node.methods.iter().find(|x| {
            return x.function_signature.name.id == CONSTRUCTOR_METHOD_NAME;
        });

        if constructor_method.is_some() {
            let function_signature = &constructor_method.unwrap().as_ref().function_signature;
            self.create_js_function(&function_signature.arguments, *function_signature.body.clone())
        } else {
            JsFunction::empty().into()
        }
    }

    fn build_prototype_object_from_class_declaration(
        &self,
        node: &ClassDeclarationNode,
    ) -> JsObject {
        let mut prototype_object = JsObject::empty();

        for class_method in &node.methods {
            let method_value = self.create_js_function(&class_method.function_signature.arguments, *class_method.function_signature.body.clone());

            prototype_object.add_property(&class_method.function_signature.name.id, method_value.into());
            // if let AstStatement::FunctionDeclaration(method_declaration) = &class_method {
                // if method_declaration.name.id == CONSTRUCTOR_METHOD_NAME { continue; }

                // let function = self.eval_function_declaration(&method_declaration).unwrap();
                //
                // if let IdentifierNode { id, .. } = method_declaration.function_signature.name.as_ref() {
                //     prototype_object.add_property(id.as_str(), function);
                // }
            // }
        }

        prototype_object
    }

    fn eval_conditional_expression(
        &self,
        node: &ConditionalExpressionNode,
    ) -> JsValue {
        let test = self.eval_expression(&node.test);

        let branch = if test.to_bool() {
            &node.consequent
        } else {
            &node.alternative
        };

        return self.eval_expression(branch.as_ref());
    }

    fn eval_return_statement(&self, node: &ReturnStatementNode) -> JsValue {
        self.eval_expression(&node.expression)
    }

    fn set_environment(&self, environment: Environment) {
        self.environment.replace(Rc::new(RefCell::new(environment)));
    }

    fn eval_for_statement(&self, node: &ForStatementNode) {
        self.set_environment(self.create_new_environment());

        if node.init.is_some() {
            self.eval_node(&node.init.as_ref().unwrap()).unwrap();
        }

        while self
            .eval_expression(&node.test.as_ref().unwrap())
            .to_bool()
        {
            self.eval_node(&node.body.as_ref()).unwrap();
            self.eval_expression(&node.update.as_ref().unwrap());
        }

        self.pop_environment();
    }

    fn call_function(&self, callee: &AstExpression, arguments: &Vec<AstExpression>, is_new: bool) -> Result<JsValue, String> {
        // println!("call_function {callee:?}");
        let calleer = self.eval_expression(&callee);

        // println!("calleer {calleer:?}");

        if let JsValue::Object(object) = &calleer {
            if let ObjectKind::Function(function) = &object.borrow().kind {
                let mut function_execution_environment = self.create_new_environment();

                // println!("expr {callee:?}");

                if let AstExpression::MemberExpression(expr) = &callee {
                    function_execution_environment.set_context(self.eval_expression(&expr.object));
                }

                // TODO: refactor, ugly as hell
                if is_new {
                    function_execution_environment.set_context(JsObject::empty().into());
                }

                let values: Vec<JsValue> = arguments
                    .iter()
                    .map(|param| self.eval_expression(&param))
                    .collect();

                match function {
                    JsFunction::Ordinary(function) => {
                        function
                            .arguments
                            .iter()
                            .zip(arguments)
                            .for_each(|(arg, node)| {
                                let value = self.eval_expression(&node);

                                function_execution_environment
                                    .define_variable(arg.name.clone(), value, false)
                                    .unwrap();
                            });
                        self.set_environment(function_execution_environment);
                        let result = function.call(self, &values);

                        if let JsValue::Object(result_object) = result.as_ref().unwrap() {
                            let proto = object.borrow().get_prototype();

                            if let JsValue::Object(object) = proto {
                                result_object.borrow_mut().set_proto(object);
                            }
                        }

                        // println!("{result:?}");
                        self.pop_environment();
                        return result;
                    }
                    JsFunction::Native(function) => {
                        self.set_environment(function_execution_environment);
                        let result = function.call(self, &values);
                        self.pop_environment();
                        return result;
                    }
                }
            }
        }

        Err(format!("{} is not callable", calleer.get_type_as_str()))
    }

    fn eval_call_expression(&self, node: &CallExpressionNode) -> Result<JsValue, String> {
        self.call_function(node.callee.as_ref(), node.params.as_ref(), false)
        // let callee = self.eval_expression(&node.callee);
        //
        // if let JsValue::Object(object) = &callee {
        //     if let ObjectKind::Function(function) = &object.borrow().kind {
        //         let mut function_execution_environment = self.create_new_environment();
        //
        //         if let AstExpression::MemberExpression(expr) = node.callee.as_ref() {
        //             function_execution_environment.define_variable(
        //                 "this".to_string(),
        //                 self.eval_expression(&expr.object),
        //             );
        //             // function_execution_environment.print_variables();
        //         }
        //
        //         let values: Vec<JsValue> = node
        //             .params
        //             .iter()
        //             .map(|param| self.eval_expression(&param))
        //             .collect();
        //
        //         match function {
        //             JsFunction::Ordinary(function) => {
        //                 function
        //                     .arguments
        //                     .iter()
        //                     .zip(&node.params)
        //                     .for_each(|(arg, node)| {
        //                         let value = self.eval_expression(&node);
        //
        //                         function_execution_environment
        //                             .define_variable(arg.name.clone(), value)
        //                             .unwrap();
        //                     });
        //                 self.set_environment(function_execution_environment);
        //                 let result = function.call(self, &values);
        //                 // println!("{result:?}");
        //                 self.pop_environment();
        //                 return result;
        //             }
        //             JsFunction::Native(function) => {
        //                 self.set_environment(function_execution_environment);
        //                 let result = function.call(self, &values);
        //                 self.pop_environment();
        //                 return result;
        //             }
        //         }
        //     }
        // }
        //
        // Err(format!("{} is not callable", callee.get_type_as_str()))
    }

    fn create_new_environment(&self) -> Environment {
        return Environment::new(Rc::clone(&self.environment.borrow().clone()));
    }

    fn pop_environment(&self) {
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

    fn eval_function_expression(&self, node: &FunctionExpressionNode) -> Result<JsValue, String> {
        let function = self.create_js_function(&node.arguments, *node.body.clone());
        let mut object = function.to_object();
        object.add_property("prototype", JsValue::object([]));
        // object.set_prototype(JsObject::empty_ref());
        return Ok(object.to_js_value());
    }

    fn eval_function_declaration(&self, node: &FunctionDeclarationNode) -> Result<JsValue, String> {
        let js_function_value: JsValue = self.create_js_function(&node.function_signature.arguments, *node.function_signature.body.clone()).into();

        if let JsValue::Object(function) = &js_function_value {
            function.borrow_mut().set_prototype(JsObject::empty_ref());
        }

        self.environment.borrow()
            .borrow_mut()
            .define_variable(node.function_signature.name.id.clone(), js_function_value.clone().into(), false)?;
        return Ok(js_function_value);
    }

    fn create_js_function(
        &self,
        function_arguments: &Vec<FunctionArgument>,
        body: AstStatement,
    ) -> JsFunction {
        let mut arguments = Vec::with_capacity(function_arguments.len());

        for fn_arg_node in function_arguments {
            let default_value = fn_arg_node
                .default_value
                .as_ref()
                .map(|node| self.eval_expression(&node.as_ref()))
                .unwrap_or(JsValue::Undefined);

            arguments.push(JsFunctionArg {
                name: fn_arg_node.name.id.clone(),
                default_value,
            });
        }

        JsFunction::ordinary_function(
            arguments,
            Box::new(body.clone()),
            self.environment.borrow().clone()
        )
    }

    fn eval_assignment_expression(
        &self,
        node: &AssignmentExpressionNode,
    ) -> Result<JsValue, String> {
        let right_hand_value = self.eval_expression(&node.right);

        match &node.left.as_ref() {
            AstExpression::Identifier(id_node) => {
                let original_value = self
                    .environment
                    .borrow()
                    .borrow()
                    .get_variable_value(&id_node.id);

                let new_variable_value = match node.operator {
                    AssignmentOperator::AddEqual => {
                        self.add(&original_value, &right_hand_value)
                    }
                    AssignmentOperator::SubEqual => {
                        self.sub(&original_value, &right_hand_value)
                    }
                    AssignmentOperator::DivEqual => {
                        self.div(&original_value, &right_hand_value)
                    }
                    AssignmentOperator::MulEqual => {
                        self.mul(&original_value, &right_hand_value)
                    }
                    AssignmentOperator::ExponentiationEqual => {
                        self.exponentiation(&original_value, &right_hand_value)
                    }
                    AssignmentOperator::Equal => Ok(right_hand_value),
                }
                .unwrap();

                self.environment.borrow()
                    .borrow_mut()
                    .assign_variable(id_node.id.clone(), new_variable_value.clone())?;
                return Ok(new_variable_value);
            }
            AstExpression::MemberExpression(node) => {
                let object = self.eval_expression(&node.object);
                let key = self.eval_member_expression_key(&node.property, node.computed)?;

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

    fn eval_while_statement(&self, node: &WhileStatementNode) {
        while self.eval_expression(&node.condition).to_bool() {
            self.eval_node(&node.body.as_ref()).unwrap();
        }
    }

    fn eval_if_statement(&self, node: &IfStatementNode) -> Result<(), String> {
        let condition_value = self.eval_expression(&node.condition);

        if condition_value.to_bool() {
            self.eval_node(&node.then_branch.as_ref()).unwrap();
            return Ok(());
        } else if let Some(node) = node.else_branch.as_ref() {
            self.eval_node(&node).unwrap();
            return Ok(());
        }

        return Ok(());
    }

    fn eval_identifier(&self, node: &IdentifierNode) -> JsValue {
        return self
            .environment
            .borrow()
            .borrow()
            .get_variable_value(&node.id);
    }

    fn eval_variable_declaration(&self, node: &VariableDeclarationNode) -> Result<(), String> {
        let value = if let Some(value) = &node.value {
            self.eval_expression(&value.as_ref())
        } else {
            JsValue::Undefined
        };
        return self.environment
            .borrow()
            .borrow_mut()
            .define_variable(node.id.id.clone(), value, matches!(&node.kind, VariableDeclarationKind::Const));
    }

    fn eval_binary_expression(&self, node: &BinaryExpressionNode) -> Result<JsValue, String> {
        let evaluated_left_node = self.eval_expression(&node.left);
        let evaluated_right_node = self.eval_expression(&node.right);

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
                ).to_string())
            }
            BinaryOperator::Equality
            | BinaryOperator::StrictEquality
            | BinaryOperator::Inequality
            | BinaryOperator::StrictInequality => {
                match (&evaluated_left_node, &evaluated_right_node) {
                    (JsValue::Number(left_number), JsValue::Number(right_number)) => {
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
                    },
                    (JsValue::String(left_string), JsValue::String(right_string)) => {
                        let value = match node.operator {
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
                        let value = match node.operator {
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

    fn logical_or(&self, left: &JsValue, right: &JsValue) -> Result<JsValue, String> {
        if left.to_bool() {
            return Ok(left.clone());
        }
        return Ok(right.clone());
    }

    fn logical_and(&self, left: &JsValue, right: &JsValue) -> Result<JsValue, String> {
        if !left.to_bool() {
            return Ok(left.clone());
        }
        return Ok(right.clone());
    }

    fn eval_statements(&self, statements: &Vec<AstStatement>) -> Result<Option<JsValue>, String> {
        let mut result: Option<JsValue> = None;

        for statement in statements {
            result = self.eval_node(&statement)?;
        }

        return Ok(result);
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

    fn div(&self, left: &JsValue, right: &JsValue) -> Result<JsValue, String> {
        left / right
    }

    fn mul(&self, left: &JsValue, right: &JsValue) -> Result<JsValue, String> {
        left * right
    }

    fn sub(&self, left: &JsValue, right: &JsValue) -> Result<JsValue, String> {
        left - right
    }

    fn add(&self, left: &JsValue, right: &JsValue) -> Result<JsValue, String> {
        left + right
    }
}

fn get_global_environment() -> Environment {
    fn console_log(_: &Interpreter, arguments: &Vec<JsValue>) -> Result<JsValue, String> {
        let result = arguments
            .iter()
            .map(|arg| format!("{}", arg))
            .collect::<Vec<String>>()
            .join(" ");
        println!("{result}");
        return Ok(JsValue::Undefined);
    }

    fn set_prototype(
        _: &Interpreter,
        arguments: &Vec<JsValue>,
    ) -> Result<JsValue, String> {
        let target = arguments
            .get(0)
            .expect("Expected first argument to be a target");

        if let JsValue::Object(target_obj) = target {
            let prototype = arguments
                .get(1)
                .expect("Expected second argument to be a prototype object");

            if let JsValue::Object(prototype_obj) = prototype {
                target_obj
                    .borrow_mut()
                    .set_proto(prototype_obj.clone());
            } else {
                return Err(format!(
                    "Second arguments should be of type object, but got: {}",
                    target.get_type_as_str()
                ));
            }
        } else {
            return Err(format!(
                "First arguments should be of type object, but got: {}",
                target.get_type_as_str()
            ));
        }

        return Ok(JsValue::Undefined);
    }

    fn performance_now(_: &Interpreter, _: &Vec<JsValue>) -> Result<JsValue, String> {
        return Ok(JsValue::Number(
            std::time::SystemTime::now()
                .duration_since( std::time::SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_millis() as f64,
        ));
    }

    fn object_keys(_: &Interpreter, args: &Vec<JsValue>) -> Result<JsValue, String> {
        assert_eq!(args.len(), 1);

        if let JsValue::Object(object) = &args[0] {
            let keys: Vec<JsValue> = object.borrow().properties.keys().map(|x| JsValue::String(x.clone())).collect();
            return Ok(JsValue::Object(JsObject::array(keys).to_ref()));
        }

        return Err("First arguments should be an object".to_string());
    }

    fn object_values(_: &Interpreter, args: &Vec<JsValue>) -> Result<JsValue, String> {
        assert_eq!(args.len(), 1);

        if let JsValue::Object(object) = &args[0] {
            let values: Vec<JsValue> = object.borrow().properties.values().map(|x| x.clone()).collect();
            return Ok(JsValue::Object(JsObject::array(values).to_ref()));
        }

        return Err("First arguments should be an object".to_string());
    }

    fn object_entries(_: &Interpreter, args: &Vec<JsValue>) -> Result<JsValue, String> {
        assert_eq!(args.len(), 1);

        if let JsValue::Object(object) = &args[0] {
            let properties = &object.borrow().properties;
            let values: Vec<JsValue> = properties.keys()
                .zip(properties.values())
                .map(|(key, value)| {
                    JsObject::array(vec![JsValue::String(key.clone()), value.clone()]).to_js_value()
                })
                .collect();
            return Ok(JsValue::Object(JsObject::array(values).to_ref()));
        }

        return Err("First arguments should be an object".to_string());
    }

    Environment::new_with_variables([
        (
            "console".to_string(),
            (true, JsValue::object([
                ("log".to_string(), JsValue::native_function(console_log)),
            ])),
        ),
        (
            "setPrototypeOf".to_string(),
            (true, JsValue::native_function(set_prototype),)
        ),
        (
            "performance".to_string(),
            (true, JsValue::object([
                ("now".to_string(), JsValue::native_function(performance_now))
            ]),)
        ),
        (
            "Object".to_string(),
            (true, JsValue::object([
                ("keys".to_string(), JsValue::native_function(object_keys)),
                ("values".to_string(), JsValue::native_function(object_values)),
                ("entries".to_string(), JsValue::native_function(object_entries)),
            ])),
        )
    ])
}

impl Default for Interpreter {
    fn default() -> Self {
        let environment = get_global_environment();
        Self {
            environment: RefCell::new(Rc::new(RefCell::new(environment))),
        }
    }
}

pub fn eval_code(code: &str) -> JsValue {
    let interpreter = Interpreter::default();

    let ast = crate::parser::Parser::parse_code_to_ast(code)
        .expect(format!("Error occurred during parsing").as_str());

    interpreter.eval_node(&ast).unwrap().unwrap()
}

fn interpret(interpreter: &mut Interpreter, code: &str) -> JsValue {
    let ast = crate::parser::Parser::parse_code_to_ast(code)
        .expect(format!("Error occurred during parsing").as_str());

    interpreter.eval_node(&ast).unwrap().unwrap()
}

#[test]
fn get_variable_value_from_parent_environment() {
    let variable_name = "abc";
    let variable_value = JsValue::Number(123.0);

    let mut parent_env = Environment::default();
    parent_env.define_variable(variable_name.to_string(), variable_value.clone(), false).unwrap();

    let child_env = Environment::new(Rc::new(RefCell::new(parent_env)));
    let value_from_parent_env = child_env.get_variable_value(variable_name);

    assert_eq!(value_from_parent_env, variable_value);
}

#[test]
fn try_to_get_undefined_variable_from_environment() {
    let env = Environment::default();
    assert_eq!(env.get_variable_value("abc"), JsValue::Undefined);
}

#[test]
fn add_operator_works() {
    let code = "2 + 2;";
    assert_eq!(eval_code(code), JsValue::Number(4.0));
}

#[test]
fn if_operator_works_then_branch() {
    let code = "let a; if (true) { a = 5; } else { a = 10; } a;";
    assert_eq!(eval_code(code), JsValue::Number(5.0));
}

#[test]
fn if_operator_works_else_branch() {
    let code = "let a; if (false) { a = 5; } else { a = 10; } a;";
    assert_eq!(eval_code(code), JsValue::Number(10.0));
}

#[test]
fn for_loop_works() {
    let code = "
    let a = 5;

    for (let i = 1; i < 11; i+=1) {
      a *= i;
    }

    a;";

    assert_eq!(eval_code(code), JsValue::Number(18144000.0));
}

#[test]
fn while_loop_works() {
    let code = "
    let a = 0;
    let i = 10;

    while (i > 0) {
        a += i;
        i -=1 ;
    }

    a;";

    assert_eq!(eval_code(code), JsValue::Number(55.0));
}

#[test]
fn equality_expression_equal_works() {
    let code = "5 == 5";
    assert_eq!(eval_code(code), JsValue::Boolean(true));
}

#[test]
fn equality_expression_not_equal_works() {
    let code = "5 == 6";
    assert_eq!(eval_code(code), JsValue::Boolean(false));
}

#[test]
fn inequality_expression_equal_works() {
    let code = "5 != 5";
    assert_eq!(eval_code(code), JsValue::Boolean(false));
}

#[test]
fn inequality_expression_not_equal_works() {
    let code = "5 != 6";
    assert_eq!(eval_code(code), JsValue::Boolean(true));
}

#[test]
fn conditional_expression_equal_works() {
    let code = "true ? 1 : 2;";
    assert_eq!(eval_code(code), JsValue::Number(1.0));
}

#[test]
fn conditional_expression_not_equal_works() {
    let code = "false ? 1 : 2;";
    assert_eq!(eval_code(code), JsValue::Number(2.0));
}

#[test]
fn object_expression_works() {
    let code = "
        let a = {
            5: 2 + 3,
            \"qwe-123\": \"string prop\",
            abc: \"identifier prop\",
            [\"hello \" + 123]: \"hello 123\",
        };

        a;
    ";

    let mut interpreter = Interpreter::default();

    let expected = JsValue::object([
        ("5".to_string(), JsValue::Number(5.0)),
        (
            "qwe-123".to_string(),
            JsValue::String("string prop".to_string()),
        ),
        (
            "abc".to_string(),
            JsValue::String("identifier prop".to_string()),
        ),
        (
            "hello 123".to_string(),
            JsValue::String("hello 123".to_string()),
        ),
    ]);

    assert_eq!(interpret(&mut interpreter, code), expected);
    assert_eq!(interpret(&mut interpreter, "a[5];"), JsValue::Number(5.0));
    assert_eq!(
        interpret(&mut interpreter, "a['qwe-123'];"),
        JsValue::String("string prop".to_string())
    );
    assert_eq!(
        interpret(&mut interpreter, "a['abc'];"),
        JsValue::String("identifier prop".to_string())
    );
    assert_eq!(
        interpret(&mut interpreter, "a.abc;"),
        JsValue::String("identifier prop".to_string())
    );
    assert_eq!(
        interpret(&mut interpreter, "a['hello ' + 123];"),
        JsValue::String("hello 123".to_string())
    );
}

#[test]
fn object_function_property() {
    let code = "
        let a = {
            b: function(a,b) {
                return a * 2 + b;
            }
        };

        a.b(3, 2);
    ";
    assert_eq!(eval_code(code), JsValue::Number(8.0));
}

#[test]
fn nested_member_expression_works() {
    let code = "
    let a = {
        b: {
            c: {
                d: \"qwerty\"
            }
        }
    };
    a.b.c.d;";
    assert_eq!(eval_code(code), JsValue::String("qwerty".to_string()));
}

#[test]
fn assign_to_object_property_works() {
    let code = "
        let a = { b: 10 };
        a.b = 20;
        a.b;
    ";
    assert_eq!(eval_code(code), JsValue::Number(20.0));
}

#[test]
fn mutate_object_as_reference_works() {
    let code = "
        let a = { b: 10 };
        let c = { d: a };
        a.b = 25;
        c.d.b;
    ";
    assert_eq!(eval_code(code), JsValue::Number(25.0));
}

#[test]
fn object_method_this_expression() {
    let mut interpreter = Interpreter::default();

    let code = "
        let a = {
          abc: 10,
          getAbc: function(a, b) {
            return this.abc;
          },
          setAbc: function(newValue) {
            this.abc = newValue;
          }
        };

        a.getAbc();
    ";
    assert_eq!(interpret(&mut interpreter, code), JsValue::Number(10.0));
    assert_eq!(
        interpret(&mut interpreter, "a.setAbc(25); a.getAbc();"),
        JsValue::Number(25.0)
    );
}

#[test]
fn comparison() {
    let mut interpreter = Interpreter::default();

    assert_eq!(interpret(&mut interpreter, "'abc' === 'abc'"), JsValue::Boolean(true));
    assert_eq!(interpret(&mut interpreter, "'abc' === 'qwe'"), JsValue::Boolean(false));
    assert_eq!(interpret(&mut interpreter, "123 === 123"), JsValue::Boolean(true));
    assert_eq!(interpret(&mut interpreter, "123 === 456"), JsValue::Boolean(false));
    assert_eq!(interpret(&mut interpreter, "true === true"), JsValue::Boolean(true));
    assert_eq!(interpret(&mut interpreter, "true === false"), JsValue::Boolean(false));
    assert_eq!(interpret(&mut interpreter, "false === false"), JsValue::Boolean(true));
    assert_eq!(eval_code("let a = {}; let b = {}; a === b;"), JsValue::Boolean(false));
    assert_eq!(eval_code("let a = {}; let b = a; a === b;"), JsValue::Boolean(true));
}

#[test]
fn prototype_property_access() {
    let mut interpreter = Interpreter::default();

    let code = "
        let prototype = {
          a: 10
        };

        let target = { b: 30 };

        setPrototypeOf(target, prototype);

        target.a;
    ";
    assert_eq!(interpret(&mut interpreter, code), JsValue::Number(10.0));
}

#[test]
fn prototype_mutable_property_access() {
    let mut interpreter = Interpreter::default();

    let code = "
        let prototype = {
          a: 10
        };

        let target = { b: 30 };

        setPrototypeOf(target, prototype);

        prototype.a = 50;

        target.a;
    ";
    assert_eq!(interpret(&mut interpreter, code), JsValue::Number(50.0));
}

#[test]
fn two_objects_must_be_checked_for_equality_by_reference() {
    let code = "
       let a = { b: { c: 10 } };

       let d = {
         e: {
           f: a
         }
       };

       d.e.f == a;
    ";
    assert_eq!(eval_code(code), JsValue::Boolean(true));

    let code = "
       let a = { b: { c: 10 } };

       let d = {
         e: {
           f: {}
         }
       };

       d.e.f == a;
    ";
    assert_eq!(eval_code(code), JsValue::Boolean(false));
}

#[test]
fn simple_class_usage() {
    let code = "
       class User {
         constructor(name, age) {
            this.name = name;
            this.age = age;
         }

         getUserInformation() {
            return 'Name is ' + this.name + ', ' + this.age + ' years old';
         }
       }

       let user = new User('Anton', 26);
       user.getUserInformation();
    ";
    assert_eq!(eval_code(code), JsValue::String("Name is Anton, 26 years old".to_string()));
}

#[test]
fn class_proto_of_instance_should_be_equal_to_class_prototype() {
    let mut interpreter = Interpreter::default();

    let code = "
       class User {
         constructor(name, age) {
            this.name = name;
            this.age = age;
         }

         getUserInformation() {
            return 'Name is ' + this.name + ', ' + this.age + ' years old';
         }
       }

       let user = new User('Anton', 26);
       user.getUserInformation();
    ";
    interpret(&mut interpreter, code);
    let class = interpreter.environment.borrow().borrow().get_variable_value("User");
    let class_instance = interpreter.environment.borrow().borrow().get_variable_value("user");

    if let JsValue::Object(class_object) = &class {
        if let JsValue::Object(instance_object) = &class_instance {
            let class_prototype = class_object.borrow().get_prototype();
            let class_instance_proto = instance_object.borrow().get_proto().unwrap();

            if let JsValue::Object(class_prototype) = class_prototype {
                assert!(Rc::ptr_eq(&class_prototype, &class_instance_proto));
            }
        }
    }
}

#[test]
fn prototypes_of_instances_of_same_class_equals() {
    let mut interpreter = Interpreter::default();
    let code = "
        class A { constructor(a) { this.a = a; } }
        new A();
    ";
    let class_instance1 = interpret(&mut interpreter, code);
    let class_instance2 = interpret(&mut interpreter, "new A();");

    if let JsValue::Object(object1) = &class_instance1 {
        if let JsValue::Object(object2) = &class_instance2 {
            let prototype1 = object1.borrow().get_proto().unwrap();
            let prototype2 = object2.borrow().get_proto().unwrap();
            assert!(Rc::ptr_eq(&prototype1, &prototype2));
        }
    }
}

#[test]
fn function_constructor_as_class() {
    let code = "
        function User(name, age) {
            this.name = name;
            this.age = age;
        }

        console.log(User.prototype);

        User.prototype.getUserInformation = function() {
            return 'Name is ' + this.name + ', ' + this.age + ' years old';
        }

        let user = new User('Anton', 26);
        user.getUserInformation();
    ";
    assert_eq!(eval_code(code), JsValue::String("Name is Anton, 26 years old".to_string()));
}

#[test]
#[should_panic(expected = "Assignment to constant variable.")]
fn attempt_to_reassign_constant_variable_should_error() {
    let code = "
        const a = 5;
        a = 10;
    ";
    eval_code(code);
}
