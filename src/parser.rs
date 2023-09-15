use crate::node::*;
use crate::scanner::{Scanner, TokenKind, Token};
use ariadne::{ColorGenerator, Label, Report, ReportKind, Source};

pub struct Parser {
    prev_token: Option<Token>,
    current_token: Option<Token>,
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
    pub fn parse_code_to_ast(code: &str) -> Result<AstStatement, String> {
        let mut parser = Parser::default();
        return parser.parse(code);
    }

    pub fn parse(&mut self, source: &str) -> Result<AstStatement, String> {
        self.source = source.to_string();
        self.scanner = Scanner::new(source.to_string());

        let mut statements: Vec<AstStatement> = vec![];

        self.current_token = self.scanner.next_token();

        while let Some(token) = &self.current_token {
            if let TokenKind::Comment(_) = token.token {
                self.next_token();
                continue;
            }

            let statement = self.parse_statement().unwrap();
            statements.push(statement);
        }

        return Ok(
            AstStatement::ProgramStatement(ProgramNode { statements }),
        );
    }

    fn parse_statement(&mut self) -> Result<AstStatement, String> {
        match self.get_current_token() {
            Some(TokenKind::LetKeyword) | Some(TokenKind::ConstKeyword) => {
                self.parse_variable_declaration()
            }
            Some(TokenKind::IfKeyword) => self.parse_if_statement(),
            Some(TokenKind::OpenBrace) => self.parse_block_statement(),
            Some(TokenKind::WhileKeyword) => self.parse_while_statement(),
            Some(TokenKind::FunctionKeyword) => self.parse_function_declaration(),
            Some(TokenKind::ReturnKeyword) => self.parse_return_statement(),
            Some(TokenKind::ForKeyword) => self.parse_for_statement(),
            // Some(TokenKind::ClassKeyword) => self.parse_class_expression(),
            _ => self.parse_expression_statement(),
        }
    }

    fn parse_class_expression(&mut self) -> Result<AstExpression, String> {
        self.eat(&TokenKind::ClassKeyword);

        let class_name_identifier = self.parse_identifier()?;
        let mut extends_identifier: Option<Box<IdentifierNode>> = None;

        if let Some(TokenKind::ExtendsKeyword) = self.get_current_token() {
            self.next_token();
            let extends_identifier_candidate = self.parse_identifier().unwrap();
            extends_identifier = Some(Box::new(extends_identifier_candidate));
        }

        let class_methods: Vec<Box<ClassMethodNode>> = self
            .parse_class_body()?
            .iter_mut()
            .map(|x| Box::new(x.clone()))
            .collect();

        return Ok(
            AstExpression::ClassDeclaration(ClassDeclarationNode {
                name: Box::new(class_name_identifier),
                parent: extends_identifier,
                methods: class_methods,
            }),
        );
    }

    fn parse_class_body(&mut self) -> Result<Vec<ClassMethodNode>, String> {
        self.eat(&TokenKind::OpenBrace);

        let mut class_methods: Vec<ClassMethodNode> = vec![];

        while let Some(TokenKind::Identifier(_)) = self.get_current_token() {
            class_methods.push(self.parse_class_method()?);
        }

        self.eat(&TokenKind::CloseBrace);

        return Ok(class_methods);
    }

    fn parse_class_method(&mut self) -> Result<ClassMethodNode, String> {
        Ok(ClassMethodNode { function_signature: self.parse_function_signature()? })
    }

    fn parse_for_statement(&mut self) -> Result<AstStatement, String> {
        self.eat(&TokenKind::ForKeyword);
        self.eat(&TokenKind::OpenParen);

        let init = self.parse_statement().unwrap();
        let test = self.parse_expression().unwrap();

        self.eat(&TokenKind::Semicolon);
        let update = self.parse_expression().unwrap();

        self.eat(&TokenKind::CloseParen);
        let body = self.parse_statement().unwrap();

        return Ok(
            AstStatement::ForStatement(ForStatementNode {
                init: Some(Box::new(init)),
                test: Some(Box::new(test)),
                update: Some(Box::new(update)),
                body: Box::new(body),
            }),
        );
    }

    fn parse_return_statement(&mut self) -> Result<AstStatement, String> {
        self.eat(&TokenKind::ReturnKeyword);
        let expression = self.parse_expression().unwrap();
        self.eat_if_present(&TokenKind::Semicolon);
        return Ok(
            AstStatement::ReturnStatement(ReturnStatementNode {
                expression: Box::new(expression),
            }),
        );
    }

    fn parse_function_declaration(&mut self) -> Result<AstStatement, String> {
        self.eat(&TokenKind::FunctionKeyword);
        Ok(AstStatement::FunctionDeclaration(FunctionDeclarationNode { function_signature: self.parse_function_signature()? }))
    }

    fn parse_function_signature(&mut self) -> Result<FunctionSignature, String> {
        let function_name = self.parse_identifier().expect("Expected function name");

        self.eat(&TokenKind::OpenParen);
        let arguments =
            self.parse_comma_sequence(&TokenKind::CloseParen, &Self::parse_function_argument)?;
        self.eat(&TokenKind::CloseParen);

        let body = self.parse_statement().unwrap();

        return Ok(FunctionSignature {
            name: Box::new(function_name),
            arguments: arguments,
            body: Box::new(body),
        });
    }

    fn parse_function_argument(&mut self) -> Result<FunctionArgument, String> {
        let name = self.parse_identifier().unwrap();

        if self.is_current_token_matches(&TokenKind::Equal) {
            self.eat(&TokenKind::Equal);
            let default_value = self.parse_expression().unwrap();

            return Ok(FunctionArgument {
                name,
                default_value: Some(Box::new(default_value)),
            });
        }

        return Ok(FunctionArgument {
            name,
            default_value: None,
        });
    }

    fn parse_while_statement(&mut self) -> Result<AstStatement, String> {
        self.eat(&TokenKind::WhileKeyword);
        self.eat(&TokenKind::OpenParen);
        let condition = self.parse_expression().unwrap();
        self.eat(&TokenKind::CloseParen);
        let body = self.parse_statement().unwrap();
        return Ok(
            AstStatement::WhileStatement(WhileStatementNode {
                condition: Box::new(condition),
                body: Box::new(body),
            }),
        );
    }

    // fn consume(&mut self, node_kind: NodeKind, start_span: Span) -> Node {
    //     let node = Node {
    //         node: node_kind,
    //         start: start_span,
    //         end: self.prev_token.as_ref().unwrap().span.end,
    //     };
    //
    //     node
    // }

    fn parse_block_statement(&mut self) -> Result<AstStatement, String> {
        let mut statements: Vec<AstStatement> = vec![];

        self.eat(&TokenKind::OpenBrace);

        while let Some(token) = &self.current_token {
            if &token.token == &TokenKind::CloseBrace {
                self.eat(&TokenKind::CloseBrace);
                break;
            }

            statements.push(self.parse_statement().unwrap());
        }

        return Ok(
            AstStatement::BlockStatement(BlockStatementNode { statements }),
        );
    }

    fn parse_identifier(&mut self) -> Result<IdentifierNode, String> {
        if let Some(TokenKind::Identifier(id)) = self.get_current_token() {
            let id = id.clone();
            let token = self.get_copy_current_token();
            self.next_token();
            return Ok(IdentifierNode { id, token });
        }

        return Err(format!(
            "Expected identifier, but got {}",
            self.get_current_token().unwrap()
        ));
    }

    fn get_current_token(&self) -> Option<&TokenKind> {
        self.current_token.as_ref().map(|x| &x.token)
    }

    fn parse_variable_declaration(&mut self) -> Result<AstStatement, String> {
        let kind = match self.get_current_token() {
            Some(TokenKind::LetKeyword) => VariableDeclarationKind::Let,
            Some(TokenKind::ConstKeyword) => VariableDeclarationKind::Const,
            _ => unreachable!(),
        };

        self.next_token();

        if let Some(TokenKind::Identifier(_)) = self.get_current_token() {
            let id = self.parse_identifier()?;

            let value = if self.is_current_token_matches(&TokenKind::Equal) {
                self.next_token();
                let expression = self
                    .parse_expression()
                    .expect("Expect variable initialization expression");

                Some(Box::new(expression))
            } else {
                None
            };

            self.eat_if_present(&TokenKind::Semicolon);

            return Ok(
                AstStatement::VariableDeclaration(VariableDeclarationNode {
                    id,
                    kind,
                    value,
                }),
            );
        } else {
            return Err("Identifier is missing in variable declaration".to_string());
        }
    }

    fn parse_expression_statement(&mut self) -> Result<AstStatement, String> {
        let expression = self.parse_expression()?;

        if self.get_current_token().is_some() && self.is_current_token_matches(&TokenKind::Semicolon) {
            self.eat(&TokenKind::Semicolon);
        }

        return Ok(expression.into());
    }

    fn parse_assignment_expression(
        &mut self,
        expression: AstExpression,
    ) -> Result<AstExpression, String> {
        let mut result_expression: AstExpression = expression;

        let assignment_tokens = vec![
            &TokenKind::PlusEqual,
            &TokenKind::MinusEqual,
            &TokenKind::DivEqual,
            &TokenKind::MulEqual,
            &TokenKind::MulMulEqual,
            &TokenKind::Equal,
        ];

        while let Some(token) = self.get_current_token() {
            if !assignment_tokens.contains(&token) {
                break;
            }
            let operator = AssignmentOperator::try_from(token).unwrap();
            self.next_token();
            let right = self.parse_expression().unwrap();
            result_expression =
                AstExpression::AssignmentExpression(AssignmentExpressionNode {
                    left: Box::new(result_expression),
                    operator: operator,
                    right: Box::new(right),
                })
            ;
        }

        return Ok(result_expression);
    }

    fn parse_expression(&mut self) -> Result<AstExpression, String> {
        let expression = self.parse_logical_or_expression()?;
        let expression = self.parse_assignment_expression(expression)?;
        return self.parse_conditional_expression(expression);
    }

    fn parse_logical_or_expression(&mut self) -> Result<AstExpression, String> {
        return self.parse_binary_expression(&Self::parse_logical_and_expression, &[TokenKind::Or]);
    }

    fn parse_logical_and_expression(&mut self) -> Result<AstExpression, String> {
        return self.parse_binary_expression(&Self::parse_equality_expression, &[TokenKind::And]);
    }

    fn parse_addition_binary_expression(&mut self) -> Result<AstExpression, String> {
        return self.parse_binary_expression(
            &Self::parse_multiplicative_and_remainder_binary_expression,
            &[TokenKind::Plus, TokenKind::Minus],
        );
    }

    fn parse_multiplicative_and_remainder_binary_expression(&mut self) -> Result<AstExpression, String> {
        return self.parse_binary_expression(
            &Self::parse_exponentiation_expression,
            &[TokenKind::Mul, TokenKind::Div, TokenKind::Percent],
        );
    }

    fn parse_exponentiation_expression(&mut self) -> Result<AstExpression, String> {
        return self.parse_binary_expression(&Self::parse_primary_expression, &[TokenKind::MulMul]);
    }

    //    fn function_call_new_computed_member_access(&mut self) -> Result<Node, String> {
    //        match self.get_current_token() {
    //            Some(Token::NewKeyword) => return self.parse_new_expression(),
    //            Some(Token::OpenParen) => return self.parse_call_expression(),
    //            _ => self.parse
    //        }
    //    }

    fn parse_comparison_expression(&mut self) -> Result<AstExpression, String> {
        return self.parse_binary_expression(
            &Self::parse_addition_binary_expression,
            &[
                TokenKind::LessThan,
                TokenKind::LessThanOrEqual,
                TokenKind::MoreThan,
                TokenKind::MoreThanOrEqual,
            ],
        );
    }

    fn parse_equality_expression(&mut self) -> Result<AstExpression, String> {
        return self.parse_binary_expression(
            &Self::parse_comparison_expression,
            &[
                TokenKind::Equality,
                TokenKind::StrictEquality,
                TokenKind::Inequality,
                TokenKind::StrictInequality,
            ],
        );
    }

    fn parse_comma_sequence<T>(
        &mut self,
        stop_token: &TokenKind,
        cb: &impl Fn(&mut Self) -> Result<T, String>,
    ) -> Result<Vec<T>, String> {
        let mut sequence = vec![];
        let mut is_first = true;

        while let Some(token) = self.get_current_token() {
            if token == stop_token {
                break;
            }

            if !is_first {
                self.eat(&TokenKind::Comma);
            }

            let expr = cb(self)?;
            sequence.push(expr);
            is_first = false;
        }

        return Ok(sequence);
    }

    fn parse_binary_expression(
        &mut self,
        side_expression_fn: &impl Fn(&mut Self) -> Result<AstExpression, String>,
        tokens: &[TokenKind],
    ) -> Result<AstExpression, String> {
        let mut left = side_expression_fn(self);

        while let Some(token) = self.get_current_token() {
            if !tokens.contains(&token) {
                break;
            }
            let operator = BinaryOperator::try_from(token).unwrap();
            self.next_token();
            let right = side_expression_fn(self);
            left = Ok(
                AstExpression::BinaryExpression(BinaryExpressionNode {
                    left: Box::new(left.unwrap()),
                    operator: operator,
                    right: Box::new(right.unwrap()),
                }),
            );
        }

        return left;
    }

    fn parse_conditional_expression(
        &mut self,
        expression: AstExpression,
    ) -> Result<AstExpression, String> {
        if self.is_current_token_matches(&TokenKind::Question) {
            self.eat(&TokenKind::Question);
            let consequent = self.parse_expression()?;
            self.eat(&TokenKind::Colon);
            let alternative = self.parse_expression()?;
            return Ok(AstExpression::ConditionalExpression(
                ConditionalExpressionNode {
                    test: Box::new(expression),
                    consequent: Box::new(consequent),
                    alternative: Box::new(alternative),
                }),
            );
        }

        return Ok(expression);
    }

    fn parse_primary_expression(&mut self) -> Result<AstExpression, String> {
        match self.get_current_token() {
            Some(TokenKind::ClassKeyword) => return self.parse_class_expression(),
            Some(TokenKind::FunctionKeyword) => return self.parse_function_expression(),
            Some(TokenKind::Number(_)) => return self.parse_number_literal(),
            Some(TokenKind::String(_)) => return self.parse_string_literal(),
            Some(TokenKind::Boolean(_)) => return self.parse_bool_literal(),
            Some(TokenKind::Null) => return self.parse_null_literal(),
            Some(TokenKind::Undefined) => return self.parse_undefined_literal(),
            Some(TokenKind::OpenParen) => return self.parse_call_expression(),
            Some(TokenKind::Identifier(_)) | Some(TokenKind::ThisKeyword) => {
                return self.parse_call_expression()
            }
            Some(TokenKind::NewKeyword) => return self.parse_new_expression(),
            Some(TokenKind::OpenBrace) => return self.parse_object_literal(),
            _ => {
                let mut colors = ColorGenerator::new();
                let token = self.current_token.as_ref().unwrap();

                Report::build(ReportKind::Error, (), token.span.start.row)
                    .with_message("Incompatible types")
                    .with_label(
                        Label::new(token.span.start.row..token.span.end.row)
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

    fn parse_function_expression(&mut self) -> Result<AstExpression, String> {
        self.eat(&TokenKind::FunctionKeyword);
        self.eat(&TokenKind::OpenParen);

        let arguments =
            self.parse_comma_sequence(&TokenKind::CloseParen, &Self::parse_function_argument)?;
        self.eat(&TokenKind::CloseParen);

        let body = self.parse_statement().unwrap();

        return Ok(AstExpression::FunctionExpression(
            FunctionExpressionNode {
                arguments: arguments,
                body: Box::new(body),
            }),
        );
    }

    fn parse_this_expression(&mut self) -> Result<AstExpression, String> {
        let token = self.get_copy_current_token();
        self.eat(&TokenKind::ThisKeyword);
        return Ok(AstExpression::ThisExpression(token));
    }

    fn parse_object_literal(&mut self) -> Result<AstExpression, String> {
        let mut properties: Vec<ObjectPropertyNode> = vec![];

        self.eat(&TokenKind::OpenBrace);

        loop {
            if self.is_current_token_matches(&TokenKind::CloseBrace) {
                break;
            }

            if properties.len() != 0 {
                self.eat(&TokenKind::Comma);
            }

            if self.is_current_token_matches(&TokenKind::CloseBrace) {
                break;
            }

            properties.push(self.parse_object_property()?);
        }

        self.eat(&TokenKind::CloseBrace);

        return Ok(AstExpression::ObjectExpression(ObjectExpressionNode { properties }));
    }

    fn parse_object_property(&mut self) -> Result<ObjectPropertyNode, String> {
        let (is_computed, key) = self.parse_object_property_key()?;
        self.eat(&TokenKind::Colon);
        let value = self.parse_expression()?;

        return Ok(ObjectPropertyNode {
            computed: is_computed,
            key: Box::new(key),
            value: Box::new(value),
        });
    }

    fn parse_object_property_key(&mut self) -> Result<(bool, AstExpression), String> {
        return match &self.get_current_token() {
            Some(TokenKind::OpenSquareBracket) => {
                self.eat(&TokenKind::OpenSquareBracket);
                let expression = self.parse_expression()?;
                self.eat(&TokenKind::CloseSquareBracket);
                return Ok((true, expression));
            }
            Some(TokenKind::Identifier(_)) => Ok((false, self.parse_identifier()?.into())),
            Some(TokenKind::String(_)) => Ok((false, self.parse_string_literal()?)),
            Some(TokenKind::Number(_)) => Ok((false, self.parse_number_literal()?)),
            _ => Err(format!(
                "{} cannot be used as an object key",
                self.get_current_token().unwrap().to_keyword()
            )),
        };
    }

    fn parse_new_expression(&mut self) -> Result<AstExpression, String> {
        self.eat(&TokenKind::NewKeyword);
        let expression = self.parse_call_expression()?;

        if let AstExpression::CallExpression(expression) = expression {
            return Ok(
                AstExpression::NewExpression(NewExpressionNode {
                    callee: expression.callee,
                    arguments: expression.params,
                }),
            );
        }

        return Err("".to_string());
    }

    fn parse_call_expression(&mut self) -> Result<AstExpression, String> {
        return self.parse_call_signature();
    }

    fn parse_member_expression(&mut self) -> Result<AstExpression, String> {
        let mut literal = self.parse_literal()?;

        loop {
            match self.get_current_token() {
                Some(&TokenKind::Dot) => {
                    self.eat(&TokenKind::Dot);
                    let property = self.parse_literal()?;

                    literal = AstExpression::MemberExpression(MemberExpressionNode {
                        computed: false,
                        object: Box::new(literal),
                        property: Box::new(property),
                    });
                }
                Some(&TokenKind::OpenSquareBracket) => {
                    self.eat(&TokenKind::OpenSquareBracket);
                    let expression = self.parse_expression()?;
                    self.eat(&TokenKind::CloseSquareBracket);

                    literal = AstExpression::MemberExpression(MemberExpressionNode {
                        computed: true,
                        object: Box::new(literal),
                        property: Box::new(expression),
                    });
                }
                _ => break,
            }
            // if let Some(&Token::Dot) = self.get_current_token() {}
        }

        return Ok(literal);
    }

    fn parse_call_signature(&mut self) -> Result<AstExpression, String> {
        let literal = self.parse_member_expression()?;

        if self.is_callee(&literal) && self.is_current_token_matches(&TokenKind::OpenParen) {
            self.eat(&TokenKind::OpenParen);
            let params = self.parse_comma_sequence(&TokenKind::CloseParen, &Self::parse_expression)?;
            self.eat(&TokenKind::CloseParen);
            return Ok(
                AstExpression::CallExpression(CallExpressionNode {
                    callee: Box::new(literal),
                    params,
                }),
            );
        }

        return Ok(literal);
    }

    fn is_callee(&self, node: &AstExpression) -> bool {
        match node {
            AstExpression::Identifier(_)
            | AstExpression::MemberExpression(_)
            | AstExpression::ThisExpression(_)
            | AstExpression::FunctionExpression(_) => true,
            _ => false,
        }
    }

    fn parse_literal(&mut self) -> Result<AstExpression, String> {
        match self.get_current_token() {
            Some(TokenKind::ThisKeyword) => return self.parse_this_expression(),
            Some(TokenKind::Number(_)) => return self.parse_number_literal(),
            Some(TokenKind::String(_)) => return self.parse_string_literal(),
            Some(TokenKind::Boolean(_)) => return self.parse_bool_literal(),
            Some(TokenKind::Null) => return self.parse_null_literal(),
            Some(TokenKind::Undefined) => return self.parse_undefined_literal(),
            Some(TokenKind::Identifier(_)) => return Ok(self.parse_identifier()?.into()),
            Some(TokenKind::FunctionKeyword) => return self.parse_function_expression(),
            Some(TokenKind::OpenParen) => return self.parse_paranthesised_expression(),
            _ => unimplemented!(),
        }
    }

    fn parse_paranthesised_expression(&mut self) -> Result<AstExpression, String> {
        self.eat(&TokenKind::OpenParen);
        let expression = self.parse_expression();
        self.eat(&TokenKind::CloseParen);
        return expression;
    }

    fn parse_bool_literal(&mut self) -> Result<AstExpression, String> {
        if let Some(TokenKind::Boolean(value)) = self.get_current_token() {
            let value = if value == "true" { true } else { false };
            let token = self.get_copy_current_token();
            self.next_token();
            return Ok(AstExpression::BooleanLiteral(BooleanLiteralNode { value, token, }));
        }

        Err(format!("Expected boolean literal, but found \"{}\"", self.get_current_token().unwrap().to_keyword()).to_string())
    }

    fn get_copy_current_token(&self) -> Token {
        self.current_token.clone().unwrap()
    }

    fn parse_null_literal(&mut self) -> Result<AstExpression, String> {
        self.eat(&TokenKind::Null);
        return Ok(AstExpression::NullLiteral(self.get_copy_current_token()));
    }

    fn parse_undefined_literal(&mut self) -> Result<AstExpression, String> {
        self.eat(&TokenKind::Undefined);
        return Ok(AstExpression::UndefinedLiteral(self.get_copy_current_token()));
    }

    fn parse_string_literal(&mut self) -> Result<AstExpression, String> {
        if let Some(TokenKind::String(str)) = self.get_current_token() {
            let value = str.clone();
            let token = self.current_token.clone().unwrap();
            self.next_token();
            return Ok(AstExpression::StringLiteral(StringLiteralNode { value, token }));
        }

        return Err(format!(
            "Expected string, but got: {}",
            self.get_current_token().unwrap().to_keyword()
        ));
    }

    fn parse_number_literal(&mut self) -> Result<AstExpression, String> {
        if let Some(TokenKind::Number(number)) = self.get_current_token() {
            let value = number.clone();
            let token = self.current_token.clone().unwrap();
            self.next_token();
            return Ok(NumberLiteralNode { value, token }.into());
        }

        return Err(format!(
            "Expected number, but got: {}",
            self.get_current_token().unwrap().to_keyword()
        ));
    }

    fn next_token(&mut self) {
        self.prev_token = self.current_token.clone();
        self.current_token = self.scanner.next_token();
    }

    fn parse_if_statement(&mut self) -> Result<AstStatement, String> {
        self.eat(&TokenKind::IfKeyword);
        self.eat(&TokenKind::OpenParen);

        let condition = Box::new(self.parse_expression().expect("Error parsing if condition"));

        self.eat(&TokenKind::CloseParen);

        let then_branch = Box::new(self.parse_statement().expect("Error parsing then branch"));

        let mut else_branch: Option<Box<AstStatement>> = None;

        if self.is_current_token_matches(&TokenKind::ElseKeyword) {
            self.next_token();

            else_branch = Some(Box::new(
                self.parse_statement().expect("Error parsing else branch"),
            ));
        }

        return Ok(
            AstStatement::IfStatement(IfStatementNode {
                condition,
                then_branch,
                else_branch,
            })
        );
    }

    fn eat(&mut self, token_kind: &TokenKind) {
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

            Report::build(ReportKind::Error, (), current_token.span.start.row)
                .with_message("Unexpected token found")
                .with_label(
                    Label::new(current_token.span.start.row..current_token.span.end.row)
                        .with_message(&error_message),
                )
                .finish()
                .print(Source::from(self.source.clone()))
                .unwrap();

            panic!("{error_message}");
        }
    }

    fn is_current_token_matches(&self, token_kind: &TokenKind) -> bool {
        self.current_token.is_some() && &self.current_token.as_ref().unwrap().token == token_kind
    }

    fn eat_if_present(&mut self, token_kind: &TokenKind) {
        if self.is_current_token_matches(token_kind) {
            self.prev_token = self.current_token.clone();
            self.next_token();
        }
    }
}
