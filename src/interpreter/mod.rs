use crate::node::*;
mod environment;
use environment::Environment;
mod js_value;
pub use js_value::JsValue;

pub struct Interpreter<'a> {
    current_enironment: Environment<'a>,
}

impl<'a> Interpreter<'a> {
    pub fn eval_node(&mut self, node: &Node) -> Result<Option<JsValue>, String> {
        //    println!("eval node: {:?}", node);
        match node {
            Node::StringLiteral(value) => Ok(Some(JsValue::String(value.to_string()))),
            Node::NumberLiteral(value) => Ok(Some(JsValue::Number(*value))),
            Node::NullLiteral => Ok(Some(JsValue::Null)),
            Node::UndefinedLiteral => Ok(Some(JsValue::Undefined)),
            Node::BooleanLiteral(value) => Ok(Some(JsValue::Boolean(*value))),
            Node::VariableDeclaration(node) => match self.eval_variable_declaration(node) {
                Ok(_) => Ok(None),
                Err(error) => Err(error.clone()),
            },
            Node::BinaryExpression(node) => match self.eval_binary_expression(node) {
                Ok(value) => Ok(Some(value)),
                Err(error) => Err(error),
            },
            Node::BlockStatement(node) => match self.eval_block_statement(node) {
                Ok(value) => Ok(value),
                Err(error) => Err(error),
            },
            Node::Identifier(node) => Ok(Some(self.eval_identifier(node).unwrap())),
            Node::IfStatement(node) => {
                self.eval_if_statement(node).unwrap();
                return Ok(None);
            },
            Node::PrintStatement(node) => {
                self.eval_print_statement(node);
                return Ok(None);
            },
            Node::AssignmentExpression(node) => {
                return self.eval_assignment_expression(node).map(|value| Some(value));
            },
            Node::WhileStatement(node) => {
                self.eval_while_statement(node);
                return Ok(None);
            },
            _ => todo!(),
        }
    }

    fn eval_assignment_expression(&mut self, node: &AssignmentExpressionNode) -> Result<JsValue, String> {
        let new_variable_value = self.eval_node(&node.right).unwrap().unwrap_or(JsValue::Undefined);

        match node.left.as_ref() {
            Node::Identifier(node) => {
                self.current_enironment.assign_variable(node.id.clone(), new_variable_value.clone()).unwrap();
                return Ok(new_variable_value);
            },
            _ => todo!()
        }
    }

    fn eval_while_statement(&mut self, node: &WhileStatementNode) {
        while self.eval_node(&node.condition).unwrap().unwrap().to_bool() {
            self.eval_node(&node.body);
        }
    }

    fn eval_print_statement(&mut self, node: &PrintStatementNode) {
        let result = self.eval_node(&node.expression).unwrap();

        match result {
            Some(value) => println!("{}", value),
            None => println!("{}", JsValue::Undefined),
        }
    }

    fn eval_if_statement(&mut self, node: &IfStatementNode) -> Result<(), String> {
        let condition_value = self
            .eval_node(node.condition.as_ref())
            .expect("Error during evaluation of condition of 'if statement'")
            .unwrap_or(JsValue::Undefined);

        if condition_value.to_bool() {
            self.eval_node(node.then_branch.as_ref());
            return Ok(());
        } else if let Some(node) = node.else_branch.as_ref() {
            println!("{:?}", node);
            self.eval_node(node);
            return Ok(());
        }

        return Ok(());
    }

    fn eval_identifier(&self, node: &IdentifierNode) -> Result<JsValue, String> {
        return self
            .current_enironment
            .get_variable_value(node.id.clone())
            .ok_or("Variable not found".to_string());
    }

    fn eval_variable_declaration(&mut self, node: &VariableDeclarationNode) -> Result<(), String> {
        let value = if let Some(value) = &node.value {
            self
                .eval_node(value.as_ref())
                .expect("Error during variable value evaluation")
                .expect("No value")
        } else {
            JsValue::Undefined
        };

//        let value = self
//            .eval_node(&node.value)
//            .expect("Error during variable value evaluation")
//            .expect("No value");
        let a = node.id.clone();
        match self.current_enironment.define_variable(a, value) {
            Ok(_) => Ok(()),
            Err(_) => todo!(),
        }
        //    match self.current_enironment.define_variable(a, value) {
        //        Ok(_) => Ok(()),
        //        Err(_) => todo!(),
        //    }
        //    return self.current_enironment.define_variable(node.id, &value).map_err(|e| e.as_str());
    }

    fn eval_binary_expression(&mut self, node: &BinaryExpressionNode) -> Result<JsValue, String> {
        let evaluated_left_node = self
            .eval_node(&node.left)
            .expect("Left expression evaluation error")
            .expect("Left expression has no value");
        let evaluated_right_node = self
            .eval_node(&node.right)
            .expect("Right expression evaluation error")
            .expect("Right expression has no value");

        match node.operator {
            BinaryOperator::Add => self.add(&evaluated_left_node, &evaluated_right_node),
            BinaryOperator::Sub => self.sub(&evaluated_left_node, &evaluated_right_node),
            BinaryOperator::Div => self.div(&evaluated_left_node, &evaluated_right_node),
            BinaryOperator::Mul => self.mul(&evaluated_left_node, &evaluated_right_node),
        }
    }

    fn eval_block_statement(
        &mut self,
        node: &BlockStatementNode,
    ) -> Result<Option<JsValue>, String> {
        let mut result: Option<JsValue> = None;

        for statement in &node.statements {
            result = self.eval_node(&statement).ok().unwrap();
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
            _ => Err(format!(
                "addition of types '{}' and '{}' is not possible",
                left.get_type_as_str(),
                right.get_type_as_str()
            )),
        }
    }
}

impl Default for Interpreter<'static> {
    fn default() -> Self {
        Self {
            current_enironment: Environment::default(),
        }
    }
}

//fn identifier_evaluation() {
//    let ast = Node::BlockStatement(BlockStatementNode {
//        statements: vec![
//            Node::VariableDeclaration(VariableDeclarationNode {
//                kind: VariableDeclarationKind::Let,
//                id: "a".to_string(),
//                value: Box::new(Node::NumberLiteral(3.0)),
//            }),
//            Node::VariableDeclaration(VariableDeclarationNode {
//                kind: VariableDeclarationKind::Let,
//                id: "b".to_string(),
//                value: Box::new(Node::NumberLiteral(3.0)),
//            }),
//            Node::VariableDeclaration(VariableDeclarationNode {
//                kind: VariableDeclarationKind::Let,
//                id: "c".to_string(),
//                value: Box::new(Node::BinaryExpression(BinaryExpressionNode {
//                    left: Box::new(Node::Identifier(IdentifierNode {
//                        id: "a".to_string(),
//                    })),
//                    operator: BinaryOperator::Add,
//                    right: Box::new(Node::Identifier(IdentifierNode {
//                        id: "b".to_string(),
//                    })),
//                })),
//            }),
//            Node::Identifier(IdentifierNode {
//                id: "c".to_string(),
//            }),
//        ],
//    });
//
//    let mut interpreter = Interpreter::default();
//
//    let result = interpreter
//        .eval_node(&ast)
//        .expect("Error during evaluating node");
//
//    match result {
//        None => println!("No Value"),
//        Some(value) => println!("> {:?}", value),
//    }
//}
