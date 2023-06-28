use crate::node::*;
use crate::scanner::{Scanner, Span, Token, TokenWithLocation};
use ariadne::{sources, ColorGenerator, Fmt, Label, Report, ReportKind, Source};

pub struct Parser {
    prev_token: Option<TokenWithLocation>,
    current_token: Option<TokenWithLocation>,
    scanner: Scanner,
    source: String,
}

impl Default for Parser {
    fn default() -> Self {
        Self {
            prev_token: None,
            current_token: None,
            scanner: Scanner::new("".to_string()),
            source: "".to_string(),
        }
    }
}

impl Parser {
    pub fn parse_code_to_ast(code: &str) -> Result<Node, String> {
        let mut parser = Parser::default();
        return parser.parse(code);
    }

    pub fn parse(&mut self, source: &str) -> Result<Node, String> {
        self.source = source.to_string();
        self.scanner = Scanner::new(source.to_string());

        let mut statements: Vec<Node> = vec![];

        self.current_token = self.scanner.next_token();
        let start_span = self.get_start_span();

        while let Some(token) = &self.current_token {
            if let Token::Comment(_) = token.token {
                self.next_token();
                continue;
            }

            let statement = self.parse_statement().unwrap();
            statements.push(statement);
        }

        return Ok(self.consume(NodeKind::ProgramStatement(ProgramNode { statements }), start_span));
    }

    fn get_start_span(&mut self) -> Span {
        self.current_token.as_ref().unwrap().start
    }

    fn parse_statement(&mut self) -> Result<Node, String> {
        match self.get_current_token() {
            Some(Token::LetKeyword) | Some(Token::ConstKeyword) => {
                self.parse_variable_declaration()
            }
            Some(Token::IfKeyword) => self.parse_if_statement(),
            Some(Token::OpenBrace) => self.parse_block_statement(),
            Some(Token::PrintKeyword) => self.parse_print_statement(),
            Some(Token::WhileKeyword) => self.parse_while_statement(),
            Some(Token::FunctionKeyword) => self.parse_function_declaration(),
            Some(Token::ReturnKeyword) => self.parse_return_statement(),
            Some(Token::ForKeyword) => self.parse_for_statement(),
            Some(Token::ClassKeyword) => self.parse_class_expression(),
            _ => self.parse_expression_statement(),
        }
    }

    fn parse_class_expression(&mut self) -> Result<Node, String> {
        let start_span = self.get_start_span();
        self.eat(&Token::ClassKeyword);

        let class_name_identifier = self.parse_identifier()?;
        let mut extends_identifier: Option<Box<Node>> = None;

        if let NodeKind::Identifier(node) = &class_name_identifier.node {
            if let Some(Token::ExtendsKeyword) = self.get_current_token() {
                self.next_token();
                let extends_identifier_candidate = self.parse_identifier().unwrap();

                if let Node {
                    node: NodeKind::Identifier(node),
                    ..
                } = &extends_identifier_candidate
                {
                    extends_identifier = Some(Box::new(extends_identifier_candidate));
                }
            }
        }

        let class_methods: Vec<Box<Node>> = self
            .parse_class_body()?
            .iter_mut()
            .map(|x| Box::new(x.clone()))
            .collect();

        return Ok(self.consume(
            NodeKind::ClassDeclaration(ClassDeclarationNode {
                name: Box::new(class_name_identifier),
                parent: extends_identifier,
                methods: class_methods,
            }),
            start_span,
        ));
    }

    fn parse_class_body(&mut self) -> Result<Vec<Node>, String> {
        self.eat(&Token::OpenBrace);

        let mut class_methods: Vec<Node> = vec![];

        while let Some(Token::Identifier(_)) = self.get_current_token() {
            class_methods.push(self.parse_class_method()?);
        }

        self.eat(&Token::CloseBrace);

        return Ok(class_methods);
    }

    fn parse_class_method(&mut self) -> Result<Node, String> {
        let start_span = self.get_start_span();
        return self.parse_function_signature(start_span);
    }

    fn parse_for_statement(&mut self) -> Result<Node, String> {
        let start_span = self.get_start_span();

        self.eat(&Token::ForKeyword);
        self.eat(&Token::OpenParen);

        let init = self.parse_statement().unwrap();
        let test = self.parse_expression().unwrap();

        self.eat(&Token::Semicolon);
        let update = self.parse_expression().unwrap();

        self.eat(&Token::CloseParen);
        let body = self.parse_statement().unwrap();

        return Ok(self.consume(
            NodeKind::ForStatement(ForStatementNode {
                init: Some(Box::new(init)),
                test: Some(Box::new(test)),
                update: Some(Box::new(update)),
                body: Box::new(body),
            }),
            start_span,
        ));
    }

    fn parse_return_statement(&mut self) -> Result<Node, String> {
        let start_span = self.get_start_span();
        self.eat(&Token::ReturnKeyword);
        let expression = self.parse_expression_statement().unwrap();
        return Ok(self.consume(
            NodeKind::ReturnStatement(ReturnStatementNode {
                expression: Box::new(expression),
            }),
            start_span,
        ));
    }

    fn parse_function_declaration(&mut self) -> Result<Node, String> {
        let start_span = self.get_start_span();
        self.eat(&Token::FunctionKeyword);
        return self.parse_function_signature(start_span);
    }

    fn parse_function_signature(&mut self, start_span: Span) -> Result<Node, String> {
        let function_name = self.parse_identifier().expect("Expected function name");

        self.eat(&Token::OpenParen);
        let arguments =
            self.parse_comma_sequence(&Token::CloseParen, &Self::parse_function_argument)?;
        self.eat(&Token::CloseParen);

        let body = self.parse_statement().unwrap();

        // TODO: get rid of exhaustive check
        if let NodeKind::Identifier(id_node) = function_name.node {
            return Ok(self.consume(
                NodeKind::FunctionDeclaration(FunctionDeclarationNode {
                    name: Box::new(id_node),
                    arguments: arguments,
                    body: Box::new(body),
                }),
                start_span,
            ));
        }

        unreachable!()
    }

//    fn parse_function_arguments(&mut self, start_span: Span) -> Result<Vec<FunctionArgument>, String> {
//
//    }

    fn parse_function_argument(&mut self) -> Result<FunctionArgument, String> {
        let start_span = self.get_start_span();

        let name = self.parse_identifier().unwrap();

        if let Some(Token::Equal) = self.get_current_token() {
            self.eat(&Token::Equal);
            let default_value = self.parse_expression().unwrap();

            // TODO: exhaustive check, don't know how to return actual Node type from parse functions
            if let NodeKind::Identifier(node) = name.node {
                return Ok(FunctionArgument {
                    name: node.id,
                    default_value: Some(Box::new(default_value)),
                });
            }

            return Err("Fuck you".to_string());
        }

        // TODO: exhaustive check, don't know how to return actual Node type from parse functions
        if let NodeKind::Identifier(node) = name.node {
            return Ok(FunctionArgument {
                name: node.id,
                default_value: None,
            });
        }

        todo!()
    }

    fn parse_while_statement(&mut self) -> Result<Node, String> {
        let start_span = self.get_start_span();
        self.eat(&Token::WhileKeyword);
        self.eat(&Token::OpenParen);
        let condition = self.parse_expression().unwrap();
        self.eat(&Token::CloseParen);
        let body = self.parse_statement().unwrap();
        return Ok(self.consume(
            NodeKind::WhileStatement(WhileStatementNode {
                condition: Box::new(condition),
                body: Box::new(body),
            }),
            start_span,
        ));
    }

    fn parse_print_statement(&mut self) -> Result<Node, String> {
        let start_span = self.get_start_span();
        self.eat(&Token::PrintKeyword);

        let expression = self.parse_expression_statement().unwrap();

        return Ok(self.consume(
            NodeKind::PrintStatement(PrintStatementNode {
                expression: Box::new(expression),
            }),
            start_span,
        ));
    }

    fn consume(&self, node_kind: NodeKind, start_span: Span) -> Node {
        Node {
            node: node_kind,
            start: start_span,
            end: self.prev_token.as_ref().unwrap().end,
        }
    }

    fn parse_block_statement(&mut self) -> Result<Node, String> {
        let start_span = self.get_start_span();
        let mut statements: Vec<Node> = vec![];

        self.eat(&Token::OpenBrace);

        while let Some(token) = &self.current_token {
            if &token.token == &Token::CloseBrace {
                self.eat(&Token::CloseBrace);
                break;
            }

            statements.push(self.parse_statement().unwrap());
        }

        return Ok(self.consume(
            NodeKind::BlockStatement(BlockStatementNode { statements }),
            start_span,
        ));
    }

    fn parse_identifier(&mut self) -> Result<Node, String> {
        let start_span = self.get_start_span();

        if let Some(Token::Identifier(id)) = self.get_current_token() {
            let id = id.clone();
            self.next_token();
            return Ok(self.consume(NodeKind::Identifier(IdentifierNode { id }), start_span));
        }

        return Err(format!(
            "Expected identifier, but got {}",
            self.get_current_token().unwrap()
        ));
    }

    fn get_current_token(&self) -> Option<&Token> {
        self.current_token.as_ref().map(|x| &x.token)
    }

    fn parse_variable_declaration(&mut self) -> Result<Node, String> {
        let start_span = self.get_start_span();

        let variable_kind = match self.get_current_token() {
            Some(Token::LetKeyword) => VariableDeclarationKind::Let,
            Some(Token::ConstKeyword) => VariableDeclarationKind::Const,
            _ => unreachable!(),
        };

        self.next_token();

        if let Some(Token::Identifier(id)) = self.get_current_token() {
            let id = id.clone();
            self.next_token();

            let value = if let Some(Token::Equal) = self.get_current_token() {
                self.next_token();
                let expression = self
                    .parse_expression()
                    .expect("Expect variable initialization expression");

                Some(Box::new(expression))
            } else {
                None
            };

            self.eat_if_present(&Token::Semicolon);

            return Ok(self.consume(
                NodeKind::VariableDeclaration(VariableDeclarationNode {
                    id: id.to_string(),
                    kind: variable_kind,
                    value: value,
                }),
                start_span,
            ));
        } else {
            return Err("Identifier is missing in variable declaration".to_string());
        }
    }

    fn parse_expression_statement(&mut self) -> Result<Node, String> {
        let expression = self.parse_expression();
        self.eat_if_present(&Token::Semicolon);
        return expression;
    }

    fn parse_assignment_expression(
        &mut self,
        expression: Node,
        start_span: Span,
    ) -> Result<Node, String> {
        //        let start_span = self.get_start_span();
        let mut result_expression: Node = expression;
        //        let mut expression = self.parse_conditional_expression();

        let assignment_tokens = vec![
            &Token::PlusEqual,
            &Token::MinusEqual,
            &Token::DivEqual,
            &Token::MulEqual,
            &Token::MulMulEqual,
            &Token::Equal,
        ];

        while let Some(token) = self.get_current_token() {
            if !assignment_tokens.contains(&token) {
                break;
            }
            let operator = AssignmentOperator::try_from(token).unwrap();
            self.next_token();
            let right = self.parse_expression().unwrap();
            result_expression = self.consume(
                NodeKind::AssignmentExpression(AssignmentExpressionNode {
                    left: Box::new(result_expression),
                    operator: operator,
                    right: Box::new(right),
                }),
                start_span,
            );
        }

        return Ok(result_expression);
    }

    fn parse_expression(&mut self) -> Result<Node, String> {
        let start_span = self.get_start_span();
        let expression = self.parse_logical_or_expression()?;
        let expression = self.parse_assignment_expression(expression, start_span.clone())?;
        return self.parse_conditional_expression(expression, start_span);
    }

    fn parse_logical_or_expression(&mut self) -> Result<Node, String> {
        return self.parse_binary_expression(&Self::parse_logical_and_expression, &[Token::Or]);
    }

    fn parse_logical_and_expression(&mut self) -> Result<Node, String> {
        return self.parse_binary_expression(&Self::parse_equality_expression, &[Token::And]);
    }

    fn parse_addition_binary_expression(&mut self) -> Result<Node, String> {
        return self.parse_binary_expression(
            &Self::parse_multiplicative_and_remainder_binary_expression,
            &[Token::Plus, Token::Minus],
        );
    }

    fn parse_multiplicative_and_remainder_binary_expression(&mut self) -> Result<Node, String> {
        return self.parse_binary_expression(
            &Self::parse_exponentiation_expression,
            &[Token::Mul, Token::Div, Token::Percent],
        );
    }

    fn parse_exponentiation_expression(&mut self) -> Result<Node, String> {
        return self.parse_binary_expression(&Self::parse_primary_expression, &[Token::MulMul]);
    }

//    fn function_call_new_computed_member_access(&mut self) -> Result<Node, String> {
//        match self.get_current_token() {
//            Some(Token::NewKeyword) => return self.parse_new_expression(),
//            Some(Token::OpenParen) => return self.parse_call_expression(),
//            _ => self.parse
//        }
//    }

    fn parse_comparison_expression(&mut self) -> Result<Node, String> {
        return self.parse_binary_expression(
            &Self::parse_addition_binary_expression,
            &[
                Token::LessThan,
                Token::LessThanOrEqual,
                Token::MoreThan,
                Token::MoreThanOrEqual,
            ],
        );
    }

    fn parse_equality_expression(&mut self) -> Result<Node, String> {
        return self.parse_binary_expression(
            &Self::parse_comparison_expression,
            &[
                Token::Equality,
                Token::StrictEquality,
                Token::Inequality,
                Token::StrictInequality,
            ],
        );
    }

    fn parse_comma_sequence<T>(
        &mut self,
        stop_token: &Token,
        cb: &impl Fn(&mut Self) -> Result<T, String>,
    ) -> Result<Vec<T>, String> {
        let mut sequence = vec![];
        let mut is_first = true;

        while let Some(token) = self.get_current_token() {
            if token == stop_token {
                break;
            }

            if !is_first {
                self.eat(&Token::Comma);
            }

            let expr = cb(self)?;
            sequence.push(expr);
            is_first = false;
        }

        return Ok(sequence);
    }

    fn parse_binary_expression(
        &mut self,
        side_expression_fn: &impl Fn(&mut Self) -> Result<Node, String>,
        tokens: &[Token],
    ) -> Result<Node, String> {
        let start_span = self.get_start_span();

        let mut left = side_expression_fn(self);

        while let Some(token) = self.get_current_token() {
            if !tokens.contains(&token) {
                break;
            }
            let operator = BinaryOperator::try_from(token).unwrap();
            self.next_token();
            let right = side_expression_fn(self);
            left = Ok(self.consume(
                NodeKind::BinaryExpression(BinaryExpressionNode {
                    left: Box::new(left.unwrap()),
                    operator: operator,
                    right: Box::new(right.unwrap()),
                }),
                start_span,
            ));
        }

        return left;
    }

    fn parse_conditional_expression(
        &mut self,
        expression: Node,
        start_span: Span,
    ) -> Result<Node, String> {
        //        let start_span = self.get_start_span();
        //        let primary_expression = self.parse_primary_expression();

        if let Some(Token::Question) = self.get_current_token() {
            self.eat(&Token::Question);
            let consequent = self.parse_expression()?;
            self.eat(&Token::Colon);
            let alternative = self.parse_expression()?;
            return Ok(self.consume(
                NodeKind::ConditionalExpression(ConditionalExpressionNode {
                    test: Box::new(expression),
                    consequent: Box::new(consequent),
                    alternative: Box::new(alternative),
                }),
                start_span,
            ));
        }

        return Ok(expression);
    }

    fn parse_primary_expression(&mut self) -> Result<Node, String> {
        println!("parse_primary_expression {:?}", self.get_current_token());
        match self.get_current_token() {
            Some(Token::ClassKeyword) => return self.parse_class_expression(),
            Some(Token::FunctionKeyword) => return self.parse_function_expression(),
            Some(Token::Number(_)) => return self.parse_number_literal(),
            Some(Token::String(_)) => return self.parse_string_literal(),
            Some(Token::Boolean(_)) => return self.parse_bool_literal(),
            Some(Token::Null) => return self.parse_null_literal(),
            Some(Token::Undefined) => return self.parse_undefined_literal(),
            Some(Token::OpenParen) => return self.parse_call_expression(),
            Some(Token::Identifier(_)) | Some(Token::ThisKeyword) => return self.parse_call_expression(),
            Some(Token::NewKeyword) => return self.parse_new_expression(),
            Some(Token::OpenBrace) => return self.parse_object_literal(),
            _ => {
                let mut colors = ColorGenerator::new();
                let token = self.current_token.as_ref().unwrap();

                Report::build(ReportKind::Error, (), token.start.row)
                    .with_message("Incompatible types")
                    .with_label(
                        Label::new(token.start.row..token.end.row)
                            .with_message("Unexpected token")
                            .with_color(colors.next()),
                    )
                    .finish()
                    .print(Source::from(&self.source))
                    .unwrap();

                unimplemented!()
            }
        }
    }

    fn parse_function_expression(&mut self) -> Result<Node, String> {
        let start_span = self.get_start_span();

        self.eat(&Token::FunctionKeyword);

        self.eat(&Token::OpenParen);
        let arguments =
            self.parse_comma_sequence(&Token::CloseParen, &Self::parse_function_argument)?;
        self.eat(&Token::CloseParen);

        let body = self.parse_statement().unwrap();

        // TODO: get rid of exhaustive check
            return Ok(self.consume(
                NodeKind::FunctionExpression(FunctionExpressionNode {
                    arguments: arguments,
                    body: Box::new(body),
                }),
                start_span,
            ));
    }

    fn parse_this_expression(&mut self) -> Result<Node, String> {
        let start_span = self.get_start_span();
        self.eat(&Token::ThisKeyword);
        return Ok(self.consume(NodeKind::ThisExpression, start_span));
    }

    fn parse_object_literal(&mut self) -> Result<Node, String> {
        let start_span = self.get_start_span();
        let mut properties: Vec<ObjectPropertyNode> = vec![];

        self.eat(&Token::OpenBrace);

        loop {
            if let Some(Token::CloseBrace) = self.get_current_token() {
                break;
            }

            if properties.len() != 0 {
                self.eat(&Token::Comma);
            }

            if let Some(Token::CloseBrace) = self.get_current_token() {
                break;
            }

            properties.push(self.parse_object_property()?.try_into()?);
        }

        //        let props: Vec<> = properties

        self.eat(&Token::CloseBrace);

        return Ok(self.consume(
            NodeKind::ObjectExpression(ObjectExpressionNode { properties }),
            start_span,
        ));
    }

    fn parse_object_property(&mut self) -> Result<Node, String> {
        let start_span = self.get_start_span();

        let (is_computed, key) = self.parse_object_property_key()?;
        self.eat(&Token::Colon);
        let value = self.parse_expression()?;

        return Ok(self.consume(
            NodeKind::ObjectProperty(ObjectPropertyNode {
                computed: is_computed,
                key: Box::new(key),
                value: Box::new(value),
            }),
            start_span,
        ));
    }

     fn parse_object_property_key(&mut self) -> Result<(bool, Node), String> {
        return match &self.get_current_token() {
            Some(Token::OpenSquareBracket) => {
                self.eat(&Token::OpenSquareBracket);
                let expression = self.parse_expression()?;
                self.eat(&Token::CloseSquareBracket);
                return Ok((true, expression));
            },
            Some(Token::Identifier(node)) => Ok((false, self.parse_identifier()?)),
            Some(Token::String(_)) => Ok((false, self.parse_string_literal()?)),
            Some(Token::Number(_)) => Ok((false, self.parse_number_literal()?)),
            _ => Err(format!("{} cannot used as an object key", self.get_current_token().unwrap().to_keyword()))
        };
     }

    fn parse_new_expression(&mut self) -> Result<Node, String> {
        let start_span = self.get_start_span();
        self.eat(&Token::NewKeyword);
        let expression = self.parse_call_expression()?;

        if let NodeKind::CallExpression(expression) = expression.node {
            return Ok(self.consume(
                NodeKind::NewExpression(NewExpressionNode {
                    callee: expression.callee,
                    arguments: expression.params,
                }),
                start_span,
            ));
        }

        return Err("".to_string());
    }

    fn parse_call_expression(&mut self) -> Result<Node, String> {
        let start_span = self.get_start_span();
        return self.parse_call_signature(start_span);
    }

    fn parse_member_expression(&mut self) -> Result<Node, String> {
        let start_span = self.get_start_span();
        let mut literal = self.parse_literal()?;

        loop {
            match self.get_current_token() {
                Some(&Token::Dot) => {
                    self.eat(&Token::Dot);
                    let property = self.parse_literal()?;

                    literal = self.consume(
                        NodeKind::MemberExpression(MemberExpressionNode {
                            computed: false,
                            object: Box::new(literal),
                            property: Box::new(property),
                        }),
                        start_span,
                    );
                }
                Some(&Token::OpenSquareBracket) => {
                    self.eat(&Token::OpenSquareBracket);
                    let expression = self.parse_expression()?;
                    self.eat(&Token::CloseSquareBracket);

                    literal = self.consume(
                        NodeKind::MemberExpression(MemberExpressionNode {
                            computed: true,
                            object: Box::new(literal),
                            property: Box::new(expression),
                        }),
                        start_span,
                    );
                }
                _ => break,
            }
            if let Some(&Token::Dot) = self.get_current_token() {}
        }

        return Ok(literal);
    }

    fn parse_call_signature(&mut self, start_span: Span) -> Result<Node, String> {
        let literal = self.parse_member_expression()?;

        println!("parse_call_signature {:?}", literal);

        if self.is_callee(&literal.node) {
            if let Some(Token::OpenParen) = self.get_current_token() {
                self.eat(&Token::OpenParen);
                let params =
                    self.parse_comma_sequence(&Token::CloseParen, &Self::parse_expression)?;
                self.eat(&Token::CloseParen);
                return Ok(self.consume(
                    NodeKind::CallExpression(CallExpressionNode {
                        callee: Box::new(literal),
                        params,
                    }),
                    start_span,
                ));
            }
        }

        return Ok(literal);
    }

    fn is_callee(&mut self, node: &NodeKind) -> bool {
        println!("is_callee {:?}", node);
        match node {
            NodeKind::Identifier(_) | NodeKind::MemberExpression(_) | NodeKind::ThisExpression | NodeKind::FunctionExpression(_) => true,
            _ => false,
        }
    }

    fn parse_literal(&mut self) -> Result<Node, String> {
        match self.get_current_token() {
            Some(Token::ThisKeyword) => return self.parse_this_expression(),
            Some(Token::Number(_)) => return self.parse_number_literal(),
            Some(Token::String(_)) => return self.parse_string_literal(),
            Some(Token::Boolean(_)) => return self.parse_bool_literal(),
            Some(Token::Null) => return self.parse_null_literal(),
            Some(Token::Undefined) => return self.parse_undefined_literal(),
            Some(Token::Identifier(_)) => return self.parse_identifier(),
            Some(Token::FunctionKeyword) => return self.parse_function_expression(),
            Some(Token::OpenParen) => return self.parse_paranthesised_expression(),
            _ => unimplemented!(),
        }
    }

    fn parse_paranthesised_expression(&mut self) -> Result<Node, String> {
        self.eat(&Token::OpenParen);
        let expression = self.parse_expression();
        println!("parse_paranthesised_expression: {expression:?}");
        self.eat(&Token::CloseParen);
        return expression;
    }

    fn parse_bool_literal(&mut self) -> Result<Node, String> {
        let start_span = self.get_start_span();
        if let Some(Token::Boolean(value)) = self.get_current_token() {
            let value = value.clone();
            self.next_token();
            return Ok(self.consume(
                NodeKind::BooleanLiteral(if value == "true" { true } else { false }),
                start_span,
            ));
        }

        unreachable!()
    }

    fn parse_null_literal(&mut self) -> Result<Node, String> {
        let start_span = self.get_start_span();
        self.eat(&Token::Null);
        return Ok(self.consume(NodeKind::NullLiteral, start_span));
    }

    fn parse_undefined_literal(&mut self) -> Result<Node, String> {
        let start_span = self.get_start_span();
        self.eat(&Token::Undefined);
        return Ok(self.consume(NodeKind::UndefinedLiteral, start_span));
    }

    fn parse_string_literal(&mut self) -> Result<Node, String> {
        let start_span = self.get_start_span();

        if let Some(Token::String(str)) = self.get_current_token() {
            let str = str.clone();
            self.next_token();
            return Ok(self.consume(NodeKind::StringLiteral(StringLiteralNode { value: str }), start_span));
        }

        unreachable!()
    }

    fn parse_number_literal(&mut self) -> Result<Node, String> {
        let start_span = self.get_start_span();
        if let Some(Token::Number(number)) = self.current_token.as_ref().map(|x| &x.token) {
            let number = number.clone();
            self.next_token();
            return Ok(self.consume(NodeKind::NumberLiteral(number), start_span));
        }

        unreachable!()
    }

    fn next_token(&mut self) {
        self.prev_token = self.current_token.clone();
        self.current_token = self.scanner.next_token();
    }

    fn parse_if_statement(&mut self) -> Result<Node, String> {
        let start_span = self.get_start_span();
        self.eat(&Token::IfKeyword);
        self.eat(&Token::OpenParen);

        let condition = Box::new(self.parse_expression().expect("Error parsing if condition"));

        self.eat(&Token::CloseParen);

        let then_branch = Box::new(self.parse_statement().expect("Error parsing then branch"));

        let mut else_branch: Option<Box<Node>> = None;

        if let Some(Token::ElseKeyword) = self.current_token.as_ref().map(|x| &x.token) {
            self.next_token();

            else_branch = Some(Box::new(
                self.parse_statement().expect("Error parsing else branch"),
            ));
        }

        return Ok(self.consume(
            NodeKind::IfStatement(IfStatementNode {
                condition,
                then_branch,
                else_branch,
            }),
            start_span,
        ));
    }

    fn eat(&mut self, token_kind: &Token) {
        if self.get_current_token().unwrap() == token_kind {
            self.prev_token = self.current_token.clone();
            self.next_token();
        } else {
            let current_token = self.current_token.as_ref().unwrap();

            let error_message = format!(
                "Expected token \"{}\", but got: {:?}",
                token_kind.to_keyword(),
                current_token.token.to_keyword()
            );

            Report::build(ReportKind::Error, (), current_token.start.row)
                .with_message("Unexpected token found")
                .with_label(
                    Label::new(current_token.start.row..current_token.end.row)
                        .with_message(&error_message),
                )
                .finish()
                .print(Source::from(self.source.clone()))
                .unwrap();

            panic!("{error_message}");
        }
    }

    fn eat_if_present(&mut self, token_kind: &Token) {
        if self.current_token.is_some() && &self.current_token.as_ref().unwrap().token == token_kind
        {
            self.prev_token = self.current_token.clone();
            self.next_token();
        }
    }
}
